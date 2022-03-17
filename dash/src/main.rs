extern crate aareocams_net;
extern crate aareocams_scomm;
extern crate anyhow;
extern crate bincode;
extern crate serde;
extern crate sn30pro;
extern crate tokio;

use aareocams_net::Message;
use aareocams_scomm::Connection;
use anyhow::Result;
use sn30pro::Controller;
use tokio::net::TcpStream;

mod config {
    pub const ADDR: &str = "127.0.0.1:6440";
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut _controlls = Controller::new(0).await?;

    let raw_socket = TcpStream::connect(config::ADDR).await?;
    let mut connection = Connection::<Message, _>::new(raw_socket, bincode::DefaultOptions::new());

    connection.queue(&Message::DashboardDisconnect)?;
    connection.send_all().await?;

    Ok(())
}
