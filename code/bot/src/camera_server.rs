use std::{
    sync::{atomic::AtomicBool, Arc},
    thread::{self, JoinHandle},
    time::Duration,
};

use aareocams_net::{Message, VideoStreamAction, VideoStreamInfo};
use nokhwa::Camera;
use uuid::Uuid;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct CameraInterface {
    #[derivative(Debug = "ignore")]
    pub cam: Camera,
    pub id: Uuid,
    pub encoder: lvenc::Encoder,
    pub paused: bool,
}

#[derive(Debug)]
pub struct CameraServer {
    handles: Vec<JoinHandle<()>>,
    messages_send: flume::Sender<Message>,
    message_queue: flume::Receiver<Message>,
    updates_receiver: flume::Receiver<(Uuid, VideoStreamAction)>,
    /// after sending any messages at all, one unpark all threads
    updates_queue: flume::Sender<(Uuid, VideoStreamAction)>,
    kill_signal: Arc<AtomicBool>,
}

impl CameraServer {
    pub fn new() -> Self {
        let handles = vec![];
        let (messages_send, message_queue) = flume::unbounded();
        let (updates_queue, updates_receiver) = flume::unbounded();

        Self {
            handles,
            messages_send,
            message_queue,
            updates_receiver,
            updates_queue,
            kill_signal: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn feed_ctrl_msg(&mut self, id: Uuid, msg: VideoStreamAction) {
        match msg {
            VideoStreamAction::Init { dev } => {
                info!("Launching new camera worker {}, device no. {}", id, dev);
                let message_queue = self.messages_send.clone();
                let update_queue = self.updates_receiver.clone();
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
                                        message_queue
                                            .send(Message::VideoStreamData { id, packet })
                                            .unwrap();
                                    }
                                }
                                Err(read_error) => {
                                    message_queue
                                        .send(Message::VideoStreamInfo {
                                            id,
                                            action: VideoStreamInfo::ReadError {
                                                message: format!("{:#?}", read_error),
                                            },
                                        })
                                        .unwrap();
                                }
                            }
                        }
                        if kill_signal.load(std::sync::atomic::Ordering::Relaxed) {
                            // me using Iterator::<Item=WhoAsked>::find()
                            let _ = interface.cam.stop_stream();
                            break 'main;
                        }
                        match update_queue.try_recv() {
                            Ok(message) => {
                                // only pay attention if it is for us
                                if message.0 == id {
                                    match message.1 {
                                        VideoStreamAction::Init { .. } => unreachable!(
                                            "this shoulld be handled by the outer match statement"
                                        ),
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
                                    flume::TryRecvError::Disconnected => panic!(
                                        "update channel was closed before threads were shut down!"
                                    ),
                                    flume::TryRecvError::Empty => {
                                        if interface.paused {
                                            // woken up if anything important happens
                                            // do not go to sleep at all if camera needs reading
                                            // park_timeout instead of just park because this code has trust issues
                                            std::thread::park_timeout(PARK_DURATION);
                                        }
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
        self.message_queue.recv_async().await.unwrap()
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
            .store(true, std::sync::atomic::Ordering::Relaxed);
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
