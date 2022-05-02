#![feature(drain_filter)]

extern crate aareocams_core;
extern crate aareocams_net;
extern crate aareocams_scomm;
extern crate anyhow;
extern crate bincode;
extern crate flume;
extern crate iced;
extern crate iced_native;
extern crate image;
extern crate lvenc;
extern crate serde;
extern crate sn30pro;
extern crate thiserror;
extern crate tokio;
extern crate uuid;
extern crate yaml_rust;
#[macro_use]
extern crate derivative;
#[macro_use]
extern crate log;
extern crate log4rs;
extern crate rustls;

mod config;
mod dash;
mod stream;

use std::net::SocketAddrV4;

use anyhow::Result;
use dash::Dashboard;
use iced::{Application, Settings};

// mod config {
//     pub const ADDR: &str = "127.0.0.1:6440";
//     pub const JOYSTICK_ID: usize = 0;
// }

fn main() -> Result<()> {
    log4rs::init_file("config/dash-log4rs.yml", Default::default())?;

    info!("Initialized logging");

    let cfg = config::load_config("config/dash.yml")?;
    info!("Read configuration {:#?}", cfg);

    Dashboard::<SocketAddrV4>::run(Settings::with_flags((cfg.bot_addr, cfg.controller_port)))?;

    Ok(())
}
