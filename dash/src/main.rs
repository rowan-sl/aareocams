extern crate aareocams_client;
extern crate aareocams_net;
extern crate aareocams_scomm;
extern crate anyhow;
extern crate bincode;
extern crate serde;
extern crate tokio;

use aareocams_client::Connection;
use aareocams_client::Controller;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;

mod config {
    pub const ADDR: &str = "127.0.0.1:6440";
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Message {}

#[tokio::main]
async fn main() -> Result<()> {
    let _controlls = Controller::new(0).await?;
    let raw_socket = TcpStream::connect(config::ADDR).await?;
    let _connection = Connection::<Message, _>::new(raw_socket, bincode::DefaultOptions::new());
    Ok(())
}
