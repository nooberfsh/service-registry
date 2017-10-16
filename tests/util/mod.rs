use service_registry::heartbeat::Server;
use service_registry::heartbeat::heartbeat_proto::*;

pub type TestServer = Server<HeartbeatRequest, HeartbeatResponse>;

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

pub fn create_server<N: Into<String>>(n: N) -> TestServer {
    Server::<HeartbeatRequest, HeartbeatResponse>::new(n, |_| simple_heartbeat_response())
}
