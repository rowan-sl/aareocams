extern crate aareocams_net;
extern crate aareocams_scomm;
extern crate anyhow;
extern crate bincode;
extern crate serde;
extern crate tokio;

use aareocams_net::Message;
use aareocams_scomm::Connection;
use anyhow::Result;
use tokio::net::TcpListener;

mod config {
    pub const ADDR: &str = "127.0.0.1:6440";
}

#[tokio::main]
async fn main() -> Result<()> {
    let listener = TcpListener::bind(config::ADDR).await?;
    let (raw_conn, _port) = listener.accept().await?;
    let mut conn = Connection::<Message, _>::new(raw_conn, bincode::DefaultOptions::new());

    loop {
        conn.recv().await?;
        println!(
            "{:?}",
            match conn.get() {
                Some(Message::DashboardDisconnect) => {
                    println!("Dashboard disconnected, exiting");
                    break;
                }
                Some(m) => m,
                None => continue,
            }
        );
    }

    Ok(())
}
