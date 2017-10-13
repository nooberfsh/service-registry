extern crate bytes;
extern crate env_logger;
extern crate future_worker;

use std::thread;
use std::time::{Duration, Instant};
use std::net::{TcpStream, SocketAddr, SocketAddrV4};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::io::{self, Read, Write};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use bytes::{BigEndian, ByteOrder};
use protobuf::stream::CodedOutputStream;
use protobuf::core::parse_from_bytes;
use protobuf::Message;
use future_worker::{Runner, FutureWorker};

use super::{Error, Server, Hub, Target};
use super::heartbeat_proto::*;

type TestServer = Server<HeartbeatRequest, HeartbeatResponse>;

pub fn simple_heartbeat_request() -> HeartbeatRequest {
    let mut req = HeartbeatRequest::new();
    req.set_msg(1);
    req
}

pub fn simple_heartbeat_response() -> HeartbeatResponse {
    let mut rsp = HeartbeatResponse::new();
    rsp.set_msg(1);
    rsp
}

fn create_server<N: Into<String>>(n: N) -> TestServer {
    Server::<HeartbeatRequest, HeartbeatResponse>::new(n, |_| simple_heartbeat_response())
}

#[test]
fn test_server() {
    let mut server = create_server("test_server");
    server.start(10000).unwrap();

    // when server was started, then call start will return Error;
    server.start(10000).unwrap_err();
    server.start(1).unwrap_err();

    let mut server2 = create_server("test_server");
    server2.start(10000).unwrap_err();

    //wait for server thread to fully start up;
    thread::sleep(Duration::from_millis(100));

    let addr: SocketAddr = "127.0.0.1:10000".parse().unwrap();
    let mut socket = TcpStream::connect(addr.clone()).unwrap();

    let request = simple_heartbeat_request();
    let size = request.compute_size();
    // 4 bytes for tokio_io frame header;
    let mut v = vec![0; (4 + size) as usize];
    // tokio_io frame use big endian;
    BigEndian::write_u32(&mut v[0..4], size);
    {
        let mut output = CodedOutputStream::bytes(&mut v[4..]);
        request.write_to(&mut output).unwrap();
    }

    socket.write_all(&v).unwrap();
    let mut v2 = vec![];
    socket.read_to_end(&mut v2).unwrap();

    let response: HeartbeatResponse = parse_from_bytes(&v2[4..]).unwrap();
    assert_eq!(response, simple_heartbeat_response());

    // send no data;
    {
        TcpStream::connect(addr.clone()).unwrap();
    }

    // send half data, then close connection
    {
        let mut socket = TcpStream::connect(addr.clone()).unwrap();
        socket.write_all(&v[0..5]).unwrap();
    }

    // recv half and then closed
    {
        let mut socket = TcpStream::connect(addr.clone()).unwrap();
        socket.write_all(&v[0..]).unwrap();
        let mut v2 = vec![0; 4];
        socket.read_exact(&mut v2).unwrap();
    }

    // send corrupted data
    {
        let mut socket = TcpStream::connect(addr.clone()).unwrap();
        let mut v = v.clone();
        for by in v[4..].iter_mut() {
            *by = !*by;
        }
        socket.write_all(&v[0..]).unwrap();
        let mut v2 = vec![];
        socket.read_to_end(&mut v2).unwrap();
        assert_eq!(v2.len(), 0);
    }
}

#[test]
fn test_hub_target_construct() {
    let addr = "127.0.0.1:10002".parse().unwrap();
    let interval = Duration::from_secs(1);
    let timeout = Duration::from_secs(1);
    let simple = simple_heartbeat_request();
    let zero = HeartbeatRequest::new();

    let target = Target::<HeartbeatRequest, HeartbeatResponse>::new(
        &addr,
        interval,
        timeout,
        simple,
        |_, _| {},
    );
    assert!(target.is_ok());

    let target = Target::<HeartbeatRequest, HeartbeatResponse>::new(
        &addr,
        interval,
        timeout,
        zero,
        |_, _| {},
    ).unwrap_err();
    assert!(target.is_zero_payload());
}

#[test]
fn test_hub_interval() {
    let port = 10004;
    let mut server = create_server("test_hub_interval");
    server.start(port).unwrap();

    //wait for server thread to fully start up;
    thread::sleep(Duration::from_millis(10));

    let start = Instant::now();
    let addr = ("127.0.0.1:".to_string() + &format!("{}", port))
        .parse()
        .unwrap();
    let interval = Duration::from_millis(50);
    let timeout = Duration::from_millis(20);

    let hub = Hub::<HeartbeatRequest, HeartbeatResponse>::new();
    let (tx, rx) = mpsc::channel();
    let target = Target::new(
        &addr,
        interval,
        timeout,
        simple_heartbeat_request(),
        move |uuid, res| { tx.send((uuid, res)).unwrap(); },
    ).unwrap();
    let id = hub.add_target(target);
    let mut count = 0;
    while count < 6 {
        let (uuid, res) = rx.recv().unwrap();
        assert_eq!(id, uuid);
        assert_eq!(res.unwrap(), simple_heartbeat_response());
        count += 1;
    }
    assert!(start.elapsed() > Duration::from_millis(250));
    assert!(start.elapsed() < Duration::from_millis(280));
}

#[test]
fn test_hub_timeout() {
    let port = 10006;
    let mut server = Server::<HeartbeatRequest, HeartbeatResponse>::new("test_hub_timeout", |_| {
        thread::sleep(Duration::from_millis(100));
        simple_heartbeat_response()
    });
    server.start(port).unwrap();

    let addr = ("127.0.0.1:".to_string() + &format!("{}", port))
        .parse()
        .unwrap();
    let interval = Duration::from_millis(50);
    let hub = Hub::<HeartbeatRequest, HeartbeatResponse>::new();
    let (tx, rx) = mpsc::channel();

    //timeout happen.
    {
        let start = Instant::now();
        let timeout_short = Duration::from_millis(50);
        let tx_short = tx.clone();
        let target_short = Target::new(
            &addr,
            interval,
            timeout_short,
            simple_heartbeat_request(),
            move |uuid, res| { tx_short.send((uuid, res)).unwrap(); },
        ).unwrap();
        let uuid = hub.add_target(target_short);
        let (id, res) = rx.recv().unwrap();
        assert_eq!(uuid, id);
        assert!(res.unwrap_err().is_timeout());
        assert!(start.elapsed() >= Duration::from_millis(50));
        assert!(start.elapsed() < Duration::from_millis(60));
    }

    //wait for server to finish the request
    thread::sleep(Duration::from_millis(50));

    //no timeout
    {
        let start = Instant::now();
        let timeout_long = Duration::from_millis(200);
        let tx_long = tx.clone();
        let target_long = Target::new(
            &addr,
            interval,
            timeout_long,
            simple_heartbeat_request(),
            move |uuid, res| { tx_long.send((uuid, res)).unwrap(); },
        ).unwrap();
        let uuid = hub.add_target(target_long);
        let (id, res) = rx.recv().unwrap();
        assert_eq!(uuid, id);
        assert_eq!(res.unwrap(), simple_heartbeat_response());
        assert!(start.elapsed() >= Duration::from_millis(100));
        assert!(start.elapsed() < Duration::from_millis(110));
    }
}

#[test]
fn test_remove_target() {
    let port = 10008;
    let mut server = create_server("test_hub_interval");
    server.start(port).unwrap();

    //wait for server thread to fully start up;
    thread::sleep(Duration::from_millis(10));

    let start = Instant::now();
    let addr = ("127.0.0.1:".to_string() + &format!("{}", port))
        .parse()
        .unwrap();
    let interval = Duration::from_millis(50);
    let timeout = Duration::from_millis(20);

    let hub = Hub::<HeartbeatRequest, HeartbeatResponse>::new();
    let (tx, rx) = mpsc::channel();
    let target = Target::new(
        &addr,
        interval,
        timeout,
        simple_heartbeat_request(),
        move |uuid, res| { tx.send((uuid, res)).unwrap(); },
    ).unwrap();
    let id = hub.add_target(target);

    let (uuid, res) = rx.recv().unwrap();
    assert_eq!(id, uuid);
    assert_eq!(res.unwrap(), simple_heartbeat_response());

    let (uuid, res) = rx.recv().unwrap();
    assert_eq!(id, uuid);
    assert_eq!(res.unwrap(), simple_heartbeat_response());

    let target = hub.remove_target(id).unwrap();
    assert_eq!(id, target.get_id());

    let e = rx.recv_timeout(interval + Duration::from_millis(20))
        .unwrap_err();
    assert_eq!(e, RecvTimeoutError::Timeout);
}
