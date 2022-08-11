use serde_derive::{Deserialize, Serialize};
#[derive(Debug, Serialize, Deserialize)]
pub enum PingReq {
    Ping(u128),
}
#[derive(Debug, Serialize, Deserialize)]
pub enum PongRepl {
    Pong(u128),
}
