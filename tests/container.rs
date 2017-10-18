#![feature(clone_closures)]

extern crate service_registry;
extern crate protobuf;
extern crate bytes;

mod util;

use std::sync::mpsc;
use std::net::{TcpStream, SocketAddr};
use std::io::{Read, Write};
use std::thread;
use std::time::{Duration, Instant};

use protobuf::core::parse_from_bytes;
use protobuf::Message;
use protobuf::stream::CodedOutputStream;
use bytes::{BigEndian, ByteOrder};

use service_registry::{ServiceId, rpc_server};
use service_registry::container::{Container, Executor};
use service_registry::heartbeat::heartbeat_proto::*;

struct Exe;

impl Executor for Exe {
    fn service_id(&self) -> ServiceId {
        100_u64.into()
    }

    fn run(&mut self, _: u16) -> bool {
        true
    }
}

#[test]
fn test_register_and_run() {
    let port = 11_010;
    let addr = ("127.0.0.1:".to_string() + &format!("{}", port))
        .parse()
        .unwrap();
    let interval = Duration::from_secs(1);
    let gen_rsp = |_| util::simple_heartbeat_response();

    let mut container =
        Container::<HeartbeatRequest, HeartbeatResponse, Exe>::new(addr, interval, gen_rsp, Exe);


    let res = container.start();
    assert!(res.is_err());

    let register_handle = move |_| {};
    let re_register_handle = move |_| {};

    let mut server = rpc_server::create_grpc_server(port, register_handle, re_register_handle)
        .unwrap();
    server.start();


    let res = container.start();
    assert!(res.is_ok());
}

#[test]
fn test_loop() {
    let port = 11_012;
    let addr = ("127.0.0.1:".to_string() + &format!("{}", port))
        .parse()
        .unwrap();

    let (tx, rx) = mpsc::channel();
    let register_handle = move |s| tx.send(s).unwrap();
    let (re_tx, re_rx) = mpsc::channel();
    let re_register_handle = move |s| re_tx.send(s).unwrap();

    let mut server = rpc_server::create_grpc_server(port, register_handle, re_register_handle)
        .unwrap();
    server.start();
    let interval = Duration::from_secs(1);
    let gen_rsp = |_| util::simple_heartbeat_response();

    let mut container =
        Container::<HeartbeatRequest, HeartbeatResponse, Exe>::new(addr, interval, gen_rsp, Exe);

    container.start().unwrap();
    let service = rx.recv().unwrap();

    let start = Instant::now();
    let re_service = re_rx.recv().unwrap();
    assert_eq!(service, re_service);
    assert!(start.elapsed() > Duration::from_secs(1));
    assert!(start.elapsed() < Duration::from_secs(2));

    for _ in 0..20 {
        thread::sleep(Duration::from_millis(100));
        send_req(service.heartbeat_addr());
    }

    let res = re_rx.try_recv();
    assert!(res.is_err());
}

fn send_req(addr: SocketAddr) {
    let mut socket = TcpStream::connect(addr).unwrap();

    let request = util::simple_heartbeat_request();
    let size = request.compute_size();
    // 4 bytes for tokio_io frame header;
    let mut v = vec![0; (4 + size) as usize];
    // tokio_io frame use big endian;
    BigEndian::write_u32(&mut v[0..4], size);
    {
        let mut output = CodedOutputStream::bytes(&mut v[4..]);
        request.write_to(&mut output).unwrap(); // TODO error handle;
    }

    socket.write_all(&v).unwrap();
    let mut v2 = vec![];
    socket.read_to_end(&mut v2).unwrap();

    let response: HeartbeatResponse = parse_from_bytes(&v2[4..]).unwrap();
    assert_eq!(response, util::simple_heartbeat_response());
}
