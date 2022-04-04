extern crate aareocams_core;
extern crate aareocams_net;
extern crate aareocams_scomm;
extern crate adafruit_motorkit;
extern crate anyhow;
extern crate bincode;
extern crate image;
// extern crate opencv;
extern crate pretty_env_logger;
extern crate serde;
extern crate tokio;
extern crate nokhwa;
#[macro_use]
extern crate log;

use std::time::Instant;

use anyhow::Result;
use nokhwa::{Camera, CameraInfo};
use aareocams_core::{H264Decoder, H264Encoder};

// use aareocams_net::Message;
// use aareocams_scomm::Stream;
// use tokio::net::TcpListener;

// mod config {
//     pub const ADDR: &str = "127.0.0.1:6440";
// }


fn get_camera_cfgs() -> Result<Vec<CameraInfo>> {
    info!("Searching for cameras");
    let mut cam_cfgs = nokhwa::query()?;
    cam_cfgs.sort_by(|a, b| {
        a.index().cmp(&b.index())
    });
    debug!("Found cameras:");
    for cfg in &cam_cfgs {
        debug!("{}: {} -- {}", cfg.index(), cfg.human_name(), cfg.description());
    }
    Ok(cam_cfgs)
}

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::formatted_builder()
        .filter_level(log::LevelFilter::Debug)
        .init();

    info!("Initialized logging");

    let cam_cfgs = get_camera_cfgs()?;

    info!("Opening camera");
    let mut cam = Camera::new(
        cam_cfgs[0].index(),
        None,
    )?;
    info!("Starting camera stream");
    cam.open_stream()?;

    debug!("Initializing encoder and decoder");
    let mut encoder = H264Encoder::new(cam.resolution().width(), cam.resolution().height())?;
    // let mut decoder = H264Decoder::new()?;

    debug!("Reading frames");
    for i in 0..60 {
        debug!("reading image #{}", i);
        let img = cam.frame()?;
        // img.save(format!("out_{}a.png", i))?;
        let before = Instant::now();
        let bytes = encoder.encode(&img)?;
        let after = Instant::now();
        debug!("encoding took {:?}, produced {} bytes", after - before, bytes.len());
        // let decoded = decoder.decode(&bytes)?;
        // for dec_img in decoded {
        //     debug!("decoded out_{}", i);
        //     // dec_img.save(format!("out_{}b.png", i))?;
        // }
    }

    info!("Done, closing stream");
    cam.stop_stream()?;

    Ok(())

    // let listener = TcpListener::bind(config::ADDR).await?;
    // loop {
    //     info!("Listening for a new connection");
    //     let (raw_conn, _port) = listener.accept().await?;
    //     let mut conn = Stream::<Message, _>::new(raw_conn, bincode::DefaultOptions::new());

    //     loop {
    //         if let Err(e) = conn.update_loop().await {
    //             error!("{:?}", e);
    //             break;
    //         }
    //         info!(
    //             "received: {:?}",
    //             match conn.get() {
    //                 Some(Message::DashboardDisconnect) => {
    //                     info!("Dashboard disconnected");
    //                     break;
    //                 }
    //                 Some(m) => m,
    //                 None => continue,
    //             }
    //         );
    //     }
    // }
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
