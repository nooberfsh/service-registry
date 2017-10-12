use std::io;

mod server;
mod hub;
pub mod heartbeat_proto;

pub use self::server::Server;
pub use self::hub::{Target, Hub};

#[derive(Debug)]
pub enum Error {
    SerializeFailed(String),
    ZeroPayload,

    IoErr(io::Error),
    Timeout,
}

#[cfg(test)]
mod test;
