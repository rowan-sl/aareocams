extern crate aareocams_scomm;
extern crate lvenc;
extern crate serde;
extern crate tokio;
extern crate uuid;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum VideoStreamAction {
    Pause,
    Resume,
    /// initialize stream, opening the camera at device ID `dev`.
    /// all future requests should be communicated using the uuid provided as part of the main message
    Init {
        dev: usize,
    },
    /// close the stream
    Close,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum VideoStreamInfo {
    Initialized,
    InitError { message: String },
    OpenCamError { message: String },
    ReadError { message: String },
    CloseError { message: String },
}

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
}
