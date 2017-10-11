pub mod client;
pub mod server;
pub mod heartbeat_proto;

use std::net::SocketAddr;

use uuid::Uuid;

#[derive(Copy, Clone, Debug)]
struct HeartbeatItem {
    uuid: Uuid,
    addr: SocketAddr,
}

impl HeartbeatItem {
    fn new<T: Into<SocketAddr>>(uuid: Uuid, addr: T) -> Self {
        HeartbeatItem {
            uuid: uuid,
            addr: addr.into(),
        }
    }
}

pub use self::heartbeat_proto::{HeartbeatRequest, HeartbeatResponse};

pub fn default_hearbeat_request() -> HeartbeatRequest {
    let mut req = HeartbeatRequest::new();
    req.set_msg(1);
    req
}

pub fn default_hearbeat_response() -> HeartbeatResponse {
    let mut rsp = HeartbeatResponse::new();
    rsp.set_msg(1);
    rsp
}

#[cfg(test)]
mod test;
