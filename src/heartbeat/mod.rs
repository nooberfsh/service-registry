use std::io;
use std::sync::Arc;

mod server;
mod hub;
mod timer;
pub mod heartbeat_proto;

pub use self::server::Server;
pub use self::hub::{Target, TargetBuilder, Hub, HubBuilder, HubHandle};

#[derive(Debug, Clone)]
pub enum Error {
    SerializeFailed(String),
    ZeroPayload,

    IoErr(Arc<io::Error>),
    Timeout,

    HubStopped,
}

impl Error {
    pub fn is_io_error(&self) -> bool {
        match *self {
            Error::IoErr(_) => true,
            _ => false,
        }
    }

    pub fn is_timeout(&self) -> bool {
        match *self {
            Error::Timeout => true,
            _ => false,
        }
    }

    pub fn is_zero_payload(&self) -> bool {
        match *self {
            Error::ZeroPayload => true,
            _ => false,
        }
    }

    pub fn is_serialize_failed(&self) -> bool {
        match *self {
            Error::SerializeFailed(_) => true,
            _ => false,
        }
    }

    pub fn is_hub_stoped(&self) -> bool {
        match *self {
            Error::HubStopped => true,
            _ => false,
        }
    }
}
