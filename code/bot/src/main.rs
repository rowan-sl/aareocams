#![feature(drain_filter)]

extern crate aareocams_core;
extern crate aareocams_net;
extern crate aareocams_scomm;
extern crate adafruit_motorkit;
extern crate anyhow;
extern crate bincode;
extern crate image;
extern crate lvenc;
extern crate nokhwa;
extern crate pretty_env_logger;
extern crate serde;
extern crate tokio;
extern crate uuid;
#[macro_use]
extern crate derivative;
#[macro_use]
extern crate log;

use std::{
    sync::{atomic::AtomicBool, Arc},
    thread::{self, JoinHandle}, time::Duration,
};

use aareocams_net::{Message, VideoStreamAction, VideoStreamInfo};
use aareocams_scomm::Stream;
use anyhow::Result;
use nokhwa::{Camera, CameraInfo};
use tokio::{
    net::TcpListener,
    select,
    sync::{broadcast, mpsc},
};
use uuid::Uuid;

mod config {
    pub const ADDR: &str = "127.0.0.1:6440";
}

pub fn get_camera_cfgs() -> Result<Vec<CameraInfo>> {
    info!("Searching for cameras");
    let mut cam_cfgs = nokhwa::query()?;
    cam_cfgs.sort_by(|a, b| a.index().cmp(&b.index()));
    debug!("Found cameras:");
    for cfg in &cam_cfgs {
        debug!(
            "{}: {} -- {}",
            cfg.index(),
            cfg.human_name(),
            cfg.description()
        );
    }
    Ok(cam_cfgs)
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct CameraInterface {
    #[derivative(Debug = "ignore")]
    pub cam: Camera,
    pub id: Uuid,
    pub encoder: lvenc::Encoder,
    pub paused: bool,
}

impl CameraInterface {
    #[must_use]
    pub fn id(&self) -> Uuid {
        self.id
    }
}

#[derive(Debug)]
pub struct CameraServer {
    handles: Vec<JoinHandle<()>>,
    messages_send: mpsc::UnboundedSender<Message>,
    message_queue: mpsc::UnboundedReceiver<Message>,
    /// after sending any messages at all, one unpark all threads
    updates_queue: broadcast::Sender<(Uuid, VideoStreamAction)>,
    kill_signal: Arc<AtomicBool>,
}

impl CameraServer {
    pub fn new() -> Self {
        let handles = vec![];
        let (messages_send, message_queue) = mpsc::unbounded_channel();
        let (updates_queue, _) = broadcast::channel(50);

        Self {
            handles,
            messages_send,
            message_queue,
            updates_queue,
            kill_signal: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn feed_ctrl_msg(&mut self, id: Uuid, msg: VideoStreamAction) {
        match msg {
            VideoStreamAction::Init { dev } => {
                info!("Launching new camera worker {}, device no. {}", id, dev);
                let message_queue = self.messages_send.clone();
                let mut update_queue = self.updates_queue.subscribe();
                let kill_signal = self.kill_signal.clone();

                const PARK_DURATION: Duration = Duration::new(5, 0);

                self.handles.push(thread::spawn(move || {
                    let mut interface = match Camera::new(dev, None) {
                        Ok(device) => {
                            let resoultion = device.resolution();
                            let interface = CameraInterface {
                                cam: device,
                                id,
                                encoder: lvenc::Encoder::new(
                                    resoultion.width(),
                                    resoultion.height(),
                                ),
                                paused: false,
                            };
                            message_queue
                                .send(Message::VideoStreamInfo {
                                    id,
                                    action: VideoStreamInfo::Initialized,
                                })
                                .unwrap();
                            interface
                        }
                        Err(init_error) => {
                            message_queue
                                .send(Message::VideoStreamInfo {
                                    id,
                                    action: VideoStreamInfo::InitError {
                                        message: format!("{:#?}", init_error),
                                    },
                                })
                                .unwrap();
                            return;
                        }
                    };

                    if let Err(open_error) = interface.cam.open_stream() {
                        message_queue
                            .send(Message::VideoStreamInfo {
                                id,
                                action: VideoStreamInfo::OpenCamError {
                                    message: format!("{:#?}", open_error),
                                },
                            })
                            .unwrap();
                        return;
                    }

                    'main: loop {
                        if !interface.paused {
                            match interface.cam.frame() {
                                Ok(frame) => {
                                    interface.encoder.encode_frame(frame);
                                    for packet in interface.encoder.packets() {
                                        message_queue.send(Message::VideoStreamData { id, packet }).unwrap();
                                    }
                                }
                                Err(read_error) => {
                                    message_queue.send(Message::VideoStreamInfo { id, action: VideoStreamInfo::ReadError { message: format!("{:#?}", read_error) } }).unwrap();
                                }
                            }
                        }
                        if kill_signal.load(std::sync::atomic::Ordering::SeqCst) {
                            // me using Iterator::<Item=WhoAsked>::find()
                            let _ = interface.cam.stop_stream();
                            break 'main;
                        }
                        match update_queue.try_recv() {
                            Ok(message) => {
                                // only pay attention if it is for us
                                if message.0 == id {
                                    match message.1 {
                                        VideoStreamAction::Init { .. } => unreachable!("this shoulld be handled by the outer match statement"),
                                        VideoStreamAction::Close => {
                                            let _ = interface.cam.stop_stream();
                                            break 'main;
                                        }
                                        VideoStreamAction::Pause => {
                                            interface.paused = true;
                                        }
                                        VideoStreamAction::Resume => {
                                            interface.paused = false;
                                        }
                                    }
                                }
                            }
                            Err(recv_err) => {
                                match recv_err {
                                    broadcast::error::TryRecvError::Closed => panic!("update channel was closed before threads were shut down!"),
                                    broadcast::error::TryRecvError::Empty => {
                                        if interface.paused {
                                            // woken up if anything important happens
                                            // do not go to sleep at all if camera needs reading
                                            // park_timeout instead of just park because this code has trust issues
                                            std::thread::park_timeout(PARK_DURATION);
                                        }
                                    }
                                    broadcast::error::TryRecvError::Lagged(num_skipped) => {
                                        error!("camera update channel size too small! skipped {} messages. increase the buffer size!", num_skipped);
                                    }
                                }
                            }
                        }
                    }
                }));
            }
            other => {
                if let Err(e) = self.updates_queue.send((id, other)) {
                    warn!("video stream action was received, but no streams are active to receive it:\n{:#?}", e);
                }
                for handle in &mut self.handles {
                    handle.thread().unpark(); // so they actually receive the message
                }
            }
        }
    }

    pub async fn collect_message(&mut self) -> Message {
        self.message_queue.recv().await.unwrap()
    }

    pub fn clean(&mut self) {
        debug!("camera server: cleaning up thread handles");
        self.handles
            .drain_filter(|i| i.is_finished())
            .for_each(|i| {
                if let Err(thread_err) = i.join() {
                    error!(
                        "camera worker thread did not exit gracefully:\n{:#?}",
                        thread_err
                    );
                }
            });
    }
}

impl Drop for CameraServer {
    fn drop(&mut self) {
        self.kill_signal
            .store(true, std::sync::atomic::Ordering::SeqCst);
        for thread in self.handles.drain(..) {
            thread.thread().unpark();
            if let Err(thread_err) = thread.join() {
                error!(
                    "camera worker thread did not exit gracefully:\n{:#?}",
                    thread_err
                );
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::formatted_builder()
        .filter_level(log::LevelFilter::Debug)
        .init();

    info!("Initialized logging");

    let listener = TcpListener::bind(config::ADDR).await?;
    info!("Listening for a new connection");
    let (raw_conn, port) = listener.accept().await?;
    info!("connected to {}", port);
    let mut conn = Stream::<Message, _>::new(raw_conn, bincode::DefaultOptions::new());

    let mut camera_server = CameraServer::new();

    loop {
        select! {
            update_res = conn.update_loop() => {
                if let Err(e) = update_res {
                    error!("{:?}", e);
                    break;
                }
                info!(
                    "received: {:?}",
                    match conn.get() {
                        Some(Message::DashboardDisconnect) => {
                            info!("Dashboard disconnected");
                            break;
                        }
                        Some(m) => {
                            match m.clone() {
                                Message::VideoStreamCtl { id, action } => {
                                    camera_server.feed_ctrl_msg(id, action);
                                }
                                _ => {}
                            }
                            m
                        }
                        None => continue,
                    }
                );
            }
            to_send = camera_server.collect_message() => {
                conn.queue(&to_send)?;
            }
        };
    }

    Ok(())
}

/// interface to the hardware of a quadrature encoder (with index)
pub trait QIEncoderInterface {
    /// get the encoder resolution in PPR
    fn get_res(&mut self) -> usize;

    // rotational progression:
    // NOTE: i could be 1 at any point throughout, but it can only be 1 at ONE PLACE and that place stays constant
    //
    // a b i
    // 0 0 1
    // 1 0 0
    // 1 1 0
    // 0 1 0
    //
    /// reads the three channels of the encoder (a, b, index)
    fn get_raw_vals(&self) -> [bool; 3];
}

/// a simple motor controller implementation.
///
/// minimum speed is 0, maximum speed is 100, and can be made negative to reverse the motor
pub trait MotorController {
    /// inverse the motor direction
    fn inverse(&mut self);

    /// sets the number of seconds it takes to go to maximmum speed (open loop ramp rate)
    fn set_ol_ramp_rate(&mut self, rate: f32);

    /// sets the motors current speed (-100 to 100) (reversed full speed to full speed)
    fn set_speed(&mut self, speed: f32);
}
