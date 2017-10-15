use std::io;

mod server;
mod hub;
mod timer;
pub mod heartbeat_proto;

pub use self::server::Server;
pub use self::hub::{Target, Hub};

#[derive(Debug)]
pub enum Error {
    SerializeFailed(String),
    ZeroPayload,

    IoErr(io::Error),
    Timeout,

    Stopped,
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
}

#[cfg(test)]
mod test;
