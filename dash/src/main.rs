extern crate aareocams_core;
extern crate aareocams_net;
extern crate aareocams_scomm;
extern crate anyhow;
extern crate bincode;
extern crate iced;
extern crate iced_native;
extern crate serde;
extern crate sn30pro;
extern crate thiserror;
extern crate tokio;
#[macro_use]
extern crate derivative;

mod stream;
mod dash;

use anyhow::Result;
use dash::Dashboard;
use iced::{Application, Settings};

mod config {
    pub const ADDR: &str = "127.0.0.1:6440";
    pub const JOYSTICK_ID: usize = 0;
}

fn main() -> Result<()> {
    Dashboard::<_>::run(Settings::with_flags((config::ADDR, config::JOYSTICK_ID)))?;

    Ok(())
}
