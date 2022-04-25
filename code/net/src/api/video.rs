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
