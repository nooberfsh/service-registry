#![feature(fnbox)]
#![feature(conservative_impl_trait)]
#![feature(clone_closures)]

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
extern crate worker;

use std::net::{SocketAddr, IpAddr};

pub mod heartbeat;
pub mod container;
pub mod registry;
pub mod rpc_server;

mod registry_proto;
mod registry_proto_grpc;

#[derive(Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct ServiceId(pub u64);

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

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Service {
    sid: ServiceId,
    meta: String,
    host: IpAddr,
    service_port: u16,
    heartbeat_port: u16,
}

impl Service {
    pub fn service_addr(&self) -> SocketAddr {
        SocketAddr::new(self.host, self.service_port)
    }

    pub fn heartbeat_addr(&self) -> SocketAddr {
        SocketAddr::new(self.host, self.heartbeat_port)
    }

    pub fn service_id(&self) -> ServiceId {
        self.sid
    }

    pub fn meta(&self) -> &str {
        &self.meta
    }
}

#[cfg(test)]
mod test;
