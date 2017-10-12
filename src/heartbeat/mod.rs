use std::io;
//
//pub mod client;
pub mod server;
//pub mod heartbeat_proto;
pub mod hub;

#[derive(Debug)]
pub enum Error {
    SerializeFailed(String),
    ZeroPayload,

    IoErr(io::Error),
    Timeout,
}

#[cfg(test)]
mod test;
