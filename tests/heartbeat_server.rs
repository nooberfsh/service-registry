extern crate service_registry;

extern crate bytes;
extern crate env_logger;
extern crate worker;
extern crate protobuf;

mod util;

use std::thread;
use std::time::Duration;
use std::net::{TcpStream, SocketAddr};
use std::io::{Read, Write};

use bytes::{BigEndian, ByteOrder};
use protobuf::stream::CodedOutputStream;
use protobuf::core::parse_from_bytes;
use protobuf::Message;

use service_registry::heartbeat::heartbeat_proto::*;

use self::util::{simple_heartbeat_request, simple_heartbeat_response, create_server};

#[test]
fn test_server() {
    let port = 10_000;
    let mut server = create_server("test_server");
    server.start(port).unwrap();

    // when server was started, then call start will return Error;
    server.start(port).unwrap_err();
    server.start(1).unwrap_err();

    let mut server2 = create_server("test_server");
    server2.start(port).unwrap_err();

    //wait for server thread to fully start up;
    thread::sleep(Duration::from_millis(100));

    let addr: SocketAddr = ("127.0.0.1:".to_string() + &format!("{}", port))
        .parse()
        .unwrap();
    let mut socket = TcpStream::connect(addr).unwrap();

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
        TcpStream::connect(addr).unwrap();
    }

    // send half data, then close connection
    {
        let mut socket = TcpStream::connect(addr).unwrap();
        socket.write_all(&v[0..5]).unwrap();
    }

    // recv half and then closed
    {
        let mut socket = TcpStream::connect(addr).unwrap();
        socket.write_all(&v[0..]).unwrap();
        let mut v2 = vec![0; 4];
        socket.read_exact(&mut v2).unwrap();
    }

    // send corrupted data
    {
        let mut socket = TcpStream::connect(addr).unwrap();
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
