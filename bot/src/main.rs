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

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::try_init_from_env(
        env_logger::Env::new()
            .default_filter_or("TRACE")
            .default_write_style_or("AUTO"),
    )?;

    info!("Initialized logging");

    let listener = TcpListener::bind(config::ADDR).await?;
    let (raw_conn, _port) = listener.accept().await?;
    let mut conn = Stream::<Message, _>::new(raw_conn, bincode::DefaultOptions::new());

    loop {
        conn.update_loop().await?;
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
