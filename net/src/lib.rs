extern crate aareocams_scomm;
extern crate serde;
extern crate tokio;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Message {
    DashboardDisconnect,
    Click,
}
