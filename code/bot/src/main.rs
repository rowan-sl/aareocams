#![feature(drain_filter)]

extern crate aareocams_core;
extern crate aareocams_net;
extern crate aareocams_scomm;
// raspberry pi motor hat
extern crate adafruit_motorkit;
extern crate anyhow;
extern crate async_trait;
extern crate bincode;
extern crate dabus;
extern crate flume;
extern crate image;
extern crate lvenc;
extern crate nokhwa;
extern crate parking_lot;
extern crate pretty_env_logger;
extern crate serde;
extern crate tokio;
extern crate uuid;
extern crate yaml_rust;
// other raspberry pi GPIO things
extern crate rppal;
#[macro_use]
extern crate derivative;
#[macro_use]
extern crate log;

pub mod camera_server;
mod config;
mod systems;

use aareocams_net::Message;
use aareocams_scomm::Stream;
// use adafruit_motorkit::dc::DcMotor;
use anyhow::Result;
use dabus::DABus;
use nokhwa::CameraInfo;
use tokio::{net::TcpListener, select};

use systems::camera;

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

#[tokio::main]
async fn main() -> Result<()> {
    println!("Reading configuration");
    let cfg = config::load_config("config/bot.yml")?;

    println!("Initializing logging");
    pretty_env_logger::formatted_builder()
        .filter_level(log::LevelFilter::Debug)
        .init();
    info!("Initialized logging");
    info!("Read configuration {:#?}", cfg);

    info!("Initializing application bus");
    let mut bus = DABus::new();

    let _ = get_camera_cfgs()?;
    info!("Starting camera server");
    bus.register(camera::CameraSystem::new());
    let camera_update_channel = bus.fire(camera::GET_RECEIVER, ()).await?;
    // info!("Starting motor controller subsystem");
    // let mut motor_controller = adafruit_motorkit::init_pwm(None)?;
    // if let Err(e) = motor_controller.enable() {
    //     error!("Failed to initialize motor controller #0\n{:#?}", e);
    //     anyhow::bail!("Error while initializing motor controller, see logs for more info");
    // }

    //* tmp code
    // let mut stepper1 = StepperMotor::try_new(&mut motor_controller, adafruit_motorkit::Motor::Stepper1, None)?;
    // loop {
    //     stepper1.step_once(&mut motor_controller, adafruit_motorkit::stepper::StepDirection::Forward, adafruit_motorkit::stepper::StepStyle::Single)?;
    // }
    // let mut motor0 = DcMotor::try_new(&mut motor_controller, adafruit_motorkit::Motor::Motor1)?;
    // motor0.set_throttle(&mut motor_controller, 0.7)?;
    // motor0.stop(&mut motor_controller)?;
    //* end tmp code

    info!("Listening for a new connection");
    let listener = TcpListener::bind(cfg.addr).await?;
    let (raw_conn, port) = listener.accept().await?;
    info!("Connected to {}", port);
    let mut conn = Stream::<Message, _>::new(raw_conn, bincode::DefaultOptions::new());

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
                                    bus.fire(camera::FEED_CTRL_MSG, (id, action)).await?;
                                }
                                other => {
                                    error!("Unhandled message:\n{:#?}", other);
                                }
                            }
                            m
                        }
                        None => continue,
                    }
                );
            }
            to_send = camera_update_channel.recv_async() => {
                conn.queue(&to_send?)?;
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
