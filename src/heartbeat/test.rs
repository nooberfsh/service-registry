extern crate bytes;
extern crate env_logger;

use std::thread;
use std::time::{Duration, Instant};
use std::net::{TcpStream, SocketAddr, SocketAddrV4};
use std::sync::mpsc::channel;
use std::io::{self, Read, Write};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use bytes::{BigEndian, ByteOrder};
use protobuf::stream::CodedOutputStream;
use protobuf::core::parse_from_bytes;
use protobuf::Message;

use super::*;
use worker;

type TestServer = server::Server<HeartbeatRequest, HeartbeatResponse>;

fn create_server<N: Into<String>>(n: N) -> TestServer {
    server::Server::<HeartbeatRequest, HeartbeatResponse>::new(n, |_| default_hearbeat_response())
}

#[test]
fn test_server() {
    let _ = env_logger::init();

    let mut server = create_server("test-server");
    server.start(10000).unwrap();

    // when server was started, then call start will return Error;
    server.start(10000).unwrap_err();
    server.start(1).unwrap_err();

    //wait for server thread to fully start up;
    thread::sleep(Duration::from_millis(100));

    let addr: SocketAddr = "127.0.0.1:10000".parse().unwrap();
    let mut socket = TcpStream::connect(addr.clone()).unwrap();

    let request = default_hearbeat_request();
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
    assert_eq!(response, default_hearbeat_response());

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
fn test_client_runner() {
    let _ = env_logger::init();
    let mut server = create_server("test-server");
    server.start(10003).unwrap();
    //wait for server thread to fully start up;
    thread::sleep(Duration::from_millis(100));

    let mut worker = worker::Worker::new("test-worker");
    worker.start(client::HeartbeatRunner);

    let addr: SocketAddr = "127.0.0.1:10003".parse().unwrap();
    let request = default_hearbeat_request();
    let v = request.write_to_bytes().unwrap();
    let (tx, rx) = channel();
    let sender = tx.clone();
    let f = move |rsp: io::Result<Vec<u8>>| {
        let response: HeartbeatResponse = parse_from_bytes(&rsp.unwrap()).unwrap();
        assert_eq!(response, default_hearbeat_response());
        sender.send(()).unwrap();
    };
    worker.schedule(client::HeartbeatTask::new(addr.clone(), v.clone(), None, f));
    rx.recv().unwrap();

    // send multiple request
    let count = 32;
    let sum = Arc::new(AtomicUsize::new(0));
    for _ in 0..count {
        let sender = tx.clone();
        let tmp = sum.clone();
        let f = move |rsp: io::Result<Vec<u8>>| {
            tmp.fetch_add(1, Ordering::SeqCst);
            let response: HeartbeatResponse = parse_from_bytes(&rsp.unwrap()).unwrap();
            assert_eq!(response, default_hearbeat_response());
            sender.send(()).unwrap();
        };
        worker.schedule(client::HeartbeatTask::new(addr.clone(), v.clone(), None, f));
    }
    for _ in 0..count {
        rx.recv().unwrap();
    }
    assert_eq!(count, sum.load(Ordering::SeqCst));


    let now = Instant::now();
    let count = 4;
    for i in 0..count {
        let sender = tx.clone();
        let f = move |rsp: io::Result<Vec<u8>>| {
            let response: HeartbeatResponse = parse_from_bytes(&rsp.unwrap()).unwrap();
            assert_eq!(response, default_hearbeat_response());
            sender.send(()).unwrap();
        };
        let delay = Duration::from_millis(50 * i);
        worker.schedule(client::HeartbeatTask::new(
            addr.clone(),
            v.clone(),
            Some(delay),
            f,
        ));
    }
    for _ in 0..count {
        rx.recv().unwrap();
    }
    info!("elapsed: {:?}", now.elapsed());
    assert!(now.elapsed() > Duration::from_millis(150));
    assert!(now.elapsed() < Duration::from_millis(200));
}

#[test]
#[should_panic]
fn test_client_same_port() {
    let _ = env_logger::init();
    let mut s1 = create_server("test_server1");
    s1.start(10001).unwrap();
    let mut s2 = create_server("test-server2");
    s2.start(10001).unwrap();
}

#[test]
fn test_client_mix() {
    let _ = env_logger::init();
    //start 3 servers;
    // heartbeat_port
    let ports = [18001, 18002, 18003];
    let mut servers = ports
        .iter()
        .map(|&port| {
            let mut server = create_server("test-server".to_string() + &format!("{}", port));
            server.start(port).unwrap();
            server
        })
        .collect::<Vec<_>>();

    //wait for server thread to fully start up;
    thread::sleep(Duration::from_millis(100));

    let succeed_count = Arc::new(AtomicUsize::new(0));
    let failed_count = Arc::new(AtomicUsize::new(0));
    let tmp_succeed = succeed_count.clone();
    let tmp_failed = failed_count.clone();
    let interval = Duration::from_millis(50);
    let mut client = client::Client::<HeartbeatRequest, HeartbeatResponse>::new(
        "test-client",
        interval,
        move |id, rsp| match rsp {
            Ok(rsp) => {
                assert_eq!(rsp, default_hearbeat_response());
                trace!("accept rsp from {:?} succeed", id);
                tmp_succeed.fetch_add(1, Ordering::SeqCst);
            }
            Err(e) => {
                trace!("accept rsp from {:?} failed, reaseon: {:?}", id, e);
                tmp_failed.fetch_add(1, Ordering::SeqCst);
            }
        },
    );
    client.start(default_hearbeat_request());

    for port in &ports {
        let addr = SocketAddr::V4(SocketAddrV4::new("127.0.0.1".parse().unwrap(), *port));
        let _ = client.add_service(addr);
    }

    thread::sleep(Duration::from_millis(300));

    assert!(succeed_count.load(Ordering::SeqCst) >= 15);
    assert!(succeed_count.load(Ordering::SeqCst) <= 18);

    servers.pop();
    servers.pop();

    succeed_count.store(0, Ordering::SeqCst);
    thread::sleep(Duration::from_millis(60));
    assert_eq!(failed_count.load(Ordering::SeqCst), 2);

    assert!(succeed_count.load(Ordering::SeqCst) >= 1);
    assert!(succeed_count.load(Ordering::SeqCst) <= 3);

    servers.pop();
    thread::sleep(Duration::from_millis(60));
    assert_eq!(failed_count.load(Ordering::SeqCst), 3);
}
