extern crate aareocams_net;
extern crate aareocams_scomm;
extern crate anyhow;
extern crate bincode;
extern crate env_logger;
extern crate serde;
extern crate tokio;
#[macro_use]
extern crate log;

use aareocams_net::Message;
use aareocams_scomm::Stream;
use anyhow::Result;
use tokio::net::TcpListener;

mod config {
    pub const ADDR: &str = "127.0.0.1:6440";
}

pub trait Encoder {
    /// gets the range of output values
    fn range(&self) -> (usize, usize);

    /// updates the encoder, receiveing values
    /// and storing them for use in the
    /// `range` `rotation` and `net_change` functions
    fn update(&mut self);

    /// gets the current encoder rotation
    fn rotation(&self) -> usize;

    /// gets the net change in location since it was last checked
    fn net_change(&self) -> usize;
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

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::try_init_from_env(
        env_logger::Env::new()
            .default_filter_or("TRACE")
            .default_write_style_or("AUTO"),
    )?;

    info!("Initialized logging");

    let listener = TcpListener::bind(config::ADDR).await?;
    loop {
        info!("Listening for a new connection");
        let (raw_conn, _port) = listener.accept().await?;
        let mut conn = Stream::<Message, _>::new(raw_conn, bincode::DefaultOptions::new());

        loop {
            if let Err(e) = conn.update_loop().await {
                error!("{:?}", e);
                break;
            }
            info!(
                "{:?}",
                match conn.get() {
                    Some(Message::DashboardDisconnect) => {
                        info!("Dashboard disconnected");
                        break;
                    }
                    Some(m) => m,
                    None => continue,
                }
            );
        }
    }
}
