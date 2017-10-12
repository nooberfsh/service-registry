#![feature(fnbox)]
#![feature(conservative_impl_trait)]

#[macro_use]
extern crate log;
extern crate futures;
extern crate tokio_core;
extern crate tokio_io;
extern crate bytes;
extern crate uuid;
extern crate mio;
extern crate grpcio;
extern crate protobuf;
extern crate future_worker;

pub mod heartbeat;
//pub mod registry;
pub mod client;
pub mod server;
mod register_proto;
mod register_proto_grpc;

#[derive(Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct ServiceId(u64);

impl From<usize> for ServiceId {
    fn from(u: usize) -> Self {
        ServiceId(u as u64)
    }
}

impl From<u64> for ServiceId {
    fn from(u: u64) -> Self {
        ServiceId(u)
    }
}

#[cfg(test)]
mod test
