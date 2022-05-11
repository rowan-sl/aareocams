// pub mod motor;
pub mod video;

use serde::{Deserialize, Serialize};

// pub use motor::*;
pub use video::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Message {
    DashboardDisconnect,
    VideoStreamData {
        id: uuid::Uuid,
        packet: lvenc::Packet,
    },
    VideoStreamInfo {
        id: uuid::Uuid,
        action: VideoStreamInfo,
    },
    VideoStreamCtl {
        id: uuid::Uuid,
        action: VideoStreamAction,
    },
    Drive (DriveAction)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum DriveAction {
    Fwd,
    Rev,
    Stop,
}
