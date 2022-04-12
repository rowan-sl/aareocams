extern crate aareocams_scomm;
extern crate serde;
extern crate tokio;
extern crate lvenc;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Message {
    DashboardDisconnect,
    VideoStream {
        stream_id: usize,
        packet: lvenc::Packet,
    }
    // VideoStream {
    //     stream_id: usize,
    //     dimensions: (u32, u32),
    //     data: Vec<u8>,
    // }
}
