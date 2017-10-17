extern crate service_registry;

extern crate worker;
extern crate protobuf;

mod util;

use std::thread;
use std::time::{Duration, Instant};
use std::sync::mpsc::{self, RecvTimeoutError};

use protobuf::Message;

use service_registry::heartbeat::{Hub, Target, HubBuilder, TargetBuilder, Server};
use service_registry::heartbeat::heartbeat_proto::*;

use self::util::{simple_heartbeat_request, simple_heartbeat_response, create_server};

#[test]
fn test_hub_target_construct() {
    let addr = "127.0.0.1:10002".parse().unwrap();
    let interval = Duration::from_secs(1);
    let timeout = Duration::from_secs(1);
    let simple = simple_heartbeat_request();
    let zero = HeartbeatRequest::new();

    let target = TargetBuilder::<HeartbeatRequest, HeartbeatResponse>::new(&addr)
        .request(zero)
        .build()
        .unwrap_err();
    assert!(target.is_zero_payload());

    let target = TargetBuilder::<HeartbeatRequest, HeartbeatResponse>::new(&addr)
        .interval(interval)
        .timeout(timeout)
        .request(simple)
        .build()
        .unwrap();

    assert_eq!(target.get_interval(), Some(interval));
    assert_eq!(target.get_timeout(), Some(timeout));
    let payload = target.get_payload().clone().unwrap();
    assert_eq!(
        simple_heartbeat_request().write_to_bytes().unwrap(),
        payload
    );
}

#[test]
fn test_hub_construct() {
    let simple = simple_heartbeat_request();
    let zero = HeartbeatRequest::new();
    let interval = Duration::from_secs(1);
    let timeout = Duration::from_secs(1);

    let r1 = HubBuilder::<HeartbeatRequest, HeartbeatResponse>::new(simple.clone())
        .timeout(timeout)
        .interval(interval)
        .build();
    let r2 = HubBuilder::<HeartbeatRequest, HeartbeatResponse>::new(zero.clone())
        .timeout(timeout)
        .interval(interval)
        .build();
    assert!(r1.is_ok());
    assert!(r2.is_err());

    let r1 = Hub::<HeartbeatRequest, HeartbeatResponse>::new(simple);
    let r2 = Hub::<HeartbeatRequest, HeartbeatResponse>::new(zero);
    assert!(r1.is_ok());
    assert!(r2.is_err());
}


#[test]
fn test_hub_interval() {
    let port = 10_004;
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

    let hub = HubBuilder::<HeartbeatRequest, HeartbeatResponse>::new(simple_heartbeat_request())
        .timeout(timeout)
        .interval(interval)
        .build()
        .unwrap();
    let (tx, rx) = mpsc::channel();
    let target = TargetBuilder::new(&addr)
        .cb(move |uuid, res| { tx.send((uuid, res)).unwrap(); })
        .build()
        .unwrap();

    let id = hub.add_target(target);
    let mut count = 0;
    while count < 6 {
        let (uuid, res) = rx.recv().unwrap();
        assert_eq!(id, uuid);
        assert_eq!(res.unwrap(), simple_heartbeat_response());
        count += 1;
    }
    assert!(start.elapsed() > Duration::from_millis(250));
    assert!(start.elapsed() < Duration::from_millis(300));
}

#[test]
fn test_hub_timeout() {
    let port = 10_006;
    let mut server = Server::<HeartbeatRequest, HeartbeatResponse>::new("test_hub_timeout", |_| {
        thread::sleep(Duration::from_millis(100));
        simple_heartbeat_response()
    });
    server.start(port).unwrap();

    let addr = ("127.0.0.1:".to_string() + &format!("{}", port))
        .parse()
        .unwrap();
    let interval = Duration::from_millis(50);
    let timeout = Duration::from_millis(10);
    let hub = HubBuilder::<HeartbeatRequest, HeartbeatResponse>::new(simple_heartbeat_request())
        .timeout(timeout)
        .interval(interval)
        .build()
        .unwrap();
    let (tx, rx) = mpsc::channel();

    //timeout happen.
    {
        let start = Instant::now();
        let timeout_short = Duration::from_millis(50);
        let tx_short = tx.clone();
        let target_short = TargetBuilder::new(&addr)
            .interval(interval)
            .timeout(timeout_short)
            .cb(move |uuid, res| { tx_short.send((uuid, res)).unwrap(); })
            .build()
            .unwrap();
        let uuid = hub.add_target(target_short);
        let (id, res) = rx.recv().unwrap();
        assert_eq!(uuid, id);
        assert!(res.unwrap_err().is_timeout());
        assert!(start.elapsed() >= Duration::from_millis(50));
        assert!(start.elapsed() < Duration::from_millis(90));
    }

    //wait for server to finish the request
    thread::sleep(Duration::from_millis(50));

    //no timeout
    {
        let start = Instant::now();
        let timeout_long = Duration::from_millis(200);
        let tx_long = tx.clone();
        let target_long = TargetBuilder::new(&addr)
            .interval(interval)
            .timeout(timeout_long)
            .cb(move |uuid, res| { tx_long.send((uuid, res)).unwrap(); })
            .build()
            .unwrap();
        let uuid = hub.add_target(target_long);
        let (id, res) = rx.recv().unwrap();
        assert_eq!(uuid, id);
        assert_eq!(res.unwrap(), simple_heartbeat_response());
        assert!(start.elapsed() >= Duration::from_millis(100));
        assert!(start.elapsed() < Duration::from_millis(140));
    }
}

#[test]
fn test_remove_target() {
    let port = 10_008;
    let mut server = create_server("test_hub_remove_target");
    server.start(port).unwrap();

    //wait for server thread to fully start up;
    thread::sleep(Duration::from_millis(10));

    let addr = ("127.0.0.1:".to_string() + &format!("{}", port))
        .parse()
        .unwrap();
    let interval = Duration::from_millis(50);
    let timeout = Duration::from_millis(20);

    let hub = HubBuilder::<HeartbeatRequest, HeartbeatResponse>::new(simple_heartbeat_request())
        .timeout(timeout)
        .interval(interval)
        .build()
        .unwrap();
    let (tx, rx) = mpsc::channel();
    let target = TargetBuilder::new(&addr)
        .cb(move |uuid, res| { tx.send((uuid, res)).unwrap(); })
        .build()
        .unwrap();
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

#[test]
fn test_hub_handle() {
    let port = 10_010;
    let mut server = create_server("test_hub_handle");
    server.start(port).unwrap();

    //wait for server thread to fully start up;
    thread::sleep(Duration::from_millis(10));

    let addr = ("127.0.0.1:".to_string() + &format!("{}", port))
        .parse()
        .unwrap();
    let interval = Duration::from_millis(50);
    let timeout = Duration::from_millis(20);

    let hub = HubBuilder::<HeartbeatRequest, HeartbeatResponse>::new(simple_heartbeat_request())
        .timeout(timeout)
        .interval(interval)
        .build()
        .unwrap();
    let hub_handle = hub.get_handle();
    let target = Target::<HeartbeatRequest, HeartbeatResponse>::new(&addr);
    let id = hub_handle.add_target(target).unwrap();

    let target = hub_handle.remove_target(id).unwrap().unwrap();
    assert_eq!(target.get_id(), id);

    drop(hub);
    let res = hub_handle.add_target(target);
    assert!(res.is_err());
    let res = hub_handle.remove_target(id);
    assert!(res.is_err());
}

#[test]
fn test_hub_cb() {
    let port = 10_012;
    let mut server = create_server("test_hub_cb");
    server.start(port).unwrap();

    //wait for server thread to fully start up;
    thread::sleep(Duration::from_millis(10));

    let addr = ("127.0.0.1:".to_string() + &format!("{}", port))
        .parse()
        .unwrap();
    let interval = Duration::from_millis(50);
    let timeout = Duration::from_millis(200);

    let (tx1, rx1) = mpsc::channel();
    let hub = HubBuilder::<HeartbeatRequest, HeartbeatResponse>::new(simple_heartbeat_request())
        .timeout(timeout)
        .interval(interval)
        .cb(move |uuid, res| { tx1.send((uuid, res)).unwrap(); })
        .build()
        .unwrap();

    let (tx2, rx2) = mpsc::channel();
    let target = TargetBuilder::new(&addr)
        .cb(move |uuid, res| { tx2.send((uuid, res)).unwrap(); })
        .build()
        .unwrap();
    let id = hub.add_target(target);

    let (uuid1, res1) = rx1.recv().unwrap();
    let (uuid2, res2) = rx2.recv().unwrap();

    assert_eq!(uuid1, uuid2);
    assert_eq!(uuid1, id);

    assert_eq!(res1.as_ref().unwrap(), res2.as_ref().unwrap());
    assert_eq!(res1.unwrap(), simple_heartbeat_response());
}

#[test]
fn test_hub_target_payload() {
    let port = 10_014;
    let mut server = Server::<HeartbeatRequest, HeartbeatResponse>::new(
        "test_hub_target_payload",
        |res| if res.msg == 1 {
            simple_heartbeat_response()
        } else {
            let mut rsp = simple_heartbeat_response();
            rsp.set_msg(res.msg + 1);
            rsp
        },
    );
    server.start(port).unwrap();

    //wait for server thread to fully start up;
    thread::sleep(Duration::from_millis(10));

    let addr = ("127.0.0.1:".to_string() + &format!("{}", port))
        .parse()
        .unwrap();

    let interval = Duration::from_millis(50);
    let timeout = Duration::from_millis(200);
    let hub = HubBuilder::<HeartbeatRequest, HeartbeatResponse>::new(simple_heartbeat_request())
        .timeout(timeout)
        .interval(interval)
        .build()
        .unwrap();

    let (tx, rx) = mpsc::channel();
    {
        let tx = tx.clone();
        let target = TargetBuilder::new(&addr)
            .cb(move |uuid, res| { tx.send((uuid, res)).unwrap(); })
            .build()
            .unwrap();
        let id = hub.add_target(target);
        let (uuid, res) = rx.recv().unwrap();
        assert_eq!(id, uuid);
        assert_eq!(res.unwrap(), simple_heartbeat_response());
        hub.remove_target(uuid).unwrap();
    }

    {
        let mut req = simple_heartbeat_request();
        req.set_msg(10);
        let target = TargetBuilder::new(&addr)
            .cb(move |uuid, res| { tx.send((uuid, res)).unwrap(); })
            .request(req)
            .build()
            .unwrap();
        let id = hub.add_target(target);
        let (uuid, res) = rx.recv().unwrap();
        assert_eq!(id, uuid);
        assert_eq!(res.unwrap().msg, 11);
    }
}
