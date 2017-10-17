use std::sync::mpsc;
use std::sync::Arc;

use grpcio::{ChannelBuilder, Environment};

use rpc_server;
use registry_proto::*;
use registry_proto_grpc::*;

#[test]
fn test_rpc_server() {
    let (tx, rx) = mpsc::channel();
    let (re_tx, re_rx) = mpsc::channel();
    let register_handle = move |s| tx.send(s).unwrap();
    let re_register_handle = move |s| re_tx.send(s).unwrap();

    let port = 11_000;
    let mut server =
        rpc_server::create_grpc_server(port, register_handle.clone(), re_register_handle.clone())
            .unwrap();
    server.start();


    let service_id = 10;
    let addr = "127.0.0.1:".to_string() + &format!("{}", port);
    let env = Arc::new(Environment::new(4));
    let ch = ChannelBuilder::new(Arc::clone(&env)).connect(&addr);
    let client = RegisterClient::new(ch);
    let mut req = RegisterRequest::new();
    req.set_service_id(service_id);
    let rsp = client.register(req).unwrap();
    let session_id = rsp.session_id;

    assert_eq!(rsp.service_port, 20_000);
    assert_eq!(rsp.heartbeat_port, 25_000);

    let mut req = StatusRequest::new();
    req.set_heartbeat_succeed(false);
    req.set_service_succeed(false);
    req.set_session_id(session_id);

    let rsp = client.report_status(req.clone()).unwrap();
    assert_eq!(rsp.succeed, true);
    assert_eq!(rsp.service_port, 20_000 + 1);
    assert_eq!(rsp.heartbeat_port, 25_000 + 1);
    assert_eq!(rsp.session_id, session_id);

    let mut req = StatusRequest::new();
    req.set_heartbeat_succeed(true);
    req.set_service_succeed(true);
    req.set_session_id(session_id);
    let _ = client.report_status(req.clone()).unwrap();

    let service = rx.recv().unwrap();
    assert_eq!(service.service_port, 20_000 + 1);
    assert_eq!(service.heartbeat_port, 25_000 + 1);

    let rsp = client.report_status(req.clone()).unwrap();
    assert_eq!(rsp.succeed, false);

    //simulate server crash.
    drop(server);
    warn!("end drop server");
    let mut server =
        rpc_server::create_grpc_server(port, register_handle.clone(), re_register_handle.clone())
            .unwrap();
    server.start();
    let ch = ChannelBuilder::new(Arc::clone(&env)).connect(&addr);
    let client = RegisterClient::new(ch);
    let rsp = client.report_status(req.clone()).unwrap();
    assert_eq!(rsp.succeed, false);

    let mut req = ReRegisterRequest::new();
    req.set_heartbeat_port(21_000);
    req.set_service_port(22_000);

    let rsp = client.re_register(req).unwrap();
    assert_eq!(rsp.succeed, true);

    let service = re_rx.recv().unwrap();
    assert_eq!(service.heartbeat_port, 21_000);
    assert_eq!(service.service_port, 22_000);
}
