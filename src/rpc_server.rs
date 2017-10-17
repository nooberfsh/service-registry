use std::io;
use std::net::IpAddr;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering, ATOMIC_USIZE_INIT};
use std::sync::{Arc, Mutex};

use grpcio::{Environment, ServerBuilder, RpcContext, UnarySink, Error as GrpcError,
             Server as GrpcServer};
use super::registry_proto_grpc::*;
use super::registry_proto::*;
use futures::Future;

use super::{ServiceId, Service};

fn fresh_session_id() -> usize {
    static NEXT_ID: AtomicUsize = ATOMIC_USIZE_INIT;
    NEXT_ID.fetch_add(1, Ordering::SeqCst)
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Debug)]
struct SessionId(u64);

impl From<usize> for SessionId {
    fn from(u: usize) -> Self {
        SessionId(u as u64)
    }
}

impl From<u64> for SessionId {
    fn from(u: u64) -> Self {
        SessionId(u)
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
struct Session {
    session_id: SessionId,
    service_id: ServiceId,
    host: IpAddr,
    service_port: u16,
    heartbeat_port: u16,
}

impl Session {
    const DEFAULT_SERVICE_PORT: u16 = 20_000;
    const DEFAULT_HEARTBEAT_PORT: u16 = 25_000;

    fn new<T: Into<IpAddr>>(service_id: ServiceId, host: T) -> Self {
        Session {
            session_id: fresh_session_id().into(),
            service_id: service_id,
            host: host.into(),
            service_port: Self::DEFAULT_SERVICE_PORT,
            heartbeat_port: Self::DEFAULT_HEARTBEAT_PORT,
        }
    }

    fn step_service_port(&mut self) {
        self.service_port += 1;
    }

    fn step_heartbeat_port(&mut self) {
        self.heartbeat_port += 1;
    }

    fn step_both(&mut self) {
        self.step_service_port();
        self.step_heartbeat_port();
    }
}

type Sessions = Arc<Mutex<HashMap<SessionId, Session>>>;

#[derive(Clone)]
pub struct RegisterService<F1, F2> {
    sessions: Sessions,
    register_handle: F1,
    re_register_handle: F2,
}

pub fn create_grpc_server<F1, F2>(
    port: u16,
    register_handle: F1,
    re_register_handle: F2,
) -> Result<GrpcServer, GrpcError>
where
    F1: Fn(Service) + Send + Clone + 'static,
    F2: Fn(Service) + Send + Clone + 'static,
{
    let env = Arc::new(Environment::new(1));
    let register_service = RegisterService::new(register_handle, re_register_handle);
    let service = create_register(register_service);
    ServerBuilder::new(env)
        .register_service(service)
        .bind("0.0.0.0", port)
        .build()
}

impl<F1, F2> RegisterService<F1, F2>
where
    F1: Fn(Service) + Send + Clone + 'static,
    F2: Fn(Service) + Send + Clone + 'static,
{
    pub fn new(register_handle: F1, re_register_handle: F2) -> Self {
        RegisterService {
            sessions: Default::default(),
            register_handle: register_handle,
            re_register_handle: re_register_handle,
        }
    }
}

impl<F1, F2> Register for RegisterService<F1, F2>
where
    F1: Fn(Service) + Send + Clone + 'static,
    F2: Fn(Service) + Send + Clone + 'static,
{
    fn register(&self, ctx: RpcContext, req: RegisterRequest, sink: UnarySink<RegisterResponse>) {
        let host = extract_host_from_grpc_bytes(ctx.host());
        let session = Session::new(req.service_id.into(), host);
        let mut lock = self.sessions.lock().unwrap();
        debug_assert!(!lock.contains_key(&session.session_id));
        lock.insert(session.session_id, session.clone());
        let rsp = session.into();
        let f = sink.success(rsp).map_err(|e| warn!("{:?}", e));
        ctx.spawn(f);
    }

    fn report_status(&self, ctx: RpcContext, req: StatusRequest, sink: UnarySink<StatusResponse>) {
        let mut rsp = StatusResponse::new();
        let mut lock = self.sessions.lock().unwrap();
        if let Some(mut session) = lock.remove(&req.session_id.into()) {
            rsp.succeed = true;
            if req.heartbeat_succeed && req.service_succeed {
                let host = extract_host_from_grpc_bytes(ctx.host());
                let service = Service {
                    sid: session.service_id,
                    host: host,
                    service_port: session.service_port,
                    heartbeat_port: session.heartbeat_port,
                };
                (self.register_handle)(service);
            } else {
                if req.heartbeat_succeed && !req.service_succeed {
                    session.step_service_port();
                } else if !req.heartbeat_succeed && req.service_succeed {
                    session.step_heartbeat_port();
                } else {
                    session.step_both();
                }
                rsp.service_port = u32::from(session.service_port);
                rsp.heartbeat_port = u32::from(session.heartbeat_port);
                rsp.session_id = session.session_id.0;
                lock.insert(session.session_id, session);
            }
        } else {
            rsp.succeed = false;
        }
        let f = sink.success(rsp).map_err(|e| warn!("{:?}", e));
        ctx.spawn(f);
    }

    fn re_register(
        &self,
        ctx: RpcContext,
        req: ReRegisterRequest,
        sink: UnarySink<ReRegisterResponse>,
    ) {
        let host = extract_host_from_grpc_bytes(ctx.host());
        let service = Service {
            sid: req.service_id.into(),
            host: host,
            service_port: req.service_port as u16,
            heartbeat_port: req.heartbeat_port as u16,
        };
        (self.re_register_handle)(service);

        let mut rsp = ReRegisterResponse::new();
        rsp.set_succeed(true);
        rsp.set_msg("succeed".to_string());
        let f = sink.success(rsp).map_err(|e| warn!("{:?}", e));
        ctx.spawn(f);
    }
}

impl From<Session> for RegisterResponse {
    fn from(s: Session) -> Self {
        let mut rsp = RegisterResponse::new();
        rsp.set_heartbeat_port(u32::from(s.heartbeat_port));
        rsp.set_service_port(u32::from(s.service_port));
        rsp.set_session_id(s.session_id.0);
        rsp
    }
}

fn bytes_to_host(host: &[u8]) -> Result<IpAddr, io::Error> {
    if host.len() != 4 && host.len() != 16 {
        Err(io::Error::new(io::ErrorKind::Other, "invalid ip addr"))
    } else if host.len() == 4 {
        let mut a = [0; 4];
        a.copy_from_slice(host);
        Ok(a.into())
    } else if host.len() == 16 {
        let mut a = [0; 16];
        a.copy_from_slice(host);
        Ok(a.into())
    } else {
        unreachable!()
    }
}

// TODO should consider ipv6, and add test for it.
fn extract_host_from_grpc_bytes(addr: &[u8]) -> IpAddr {
    assert!(addr.contains(&b':'));
    let addr = addr.split(|c| c == &b':').nth(0).unwrap();
    let mut host = Vec::with_capacity(16);
    for t in addr.split(|c| c == &b'.') {
        let mut sum = 0;
        for (i, c) in t.iter().enumerate() {
            sum += (*c - b'0') * 10u8.pow((t.len() - i - 1) as u32);
        }
        host.push(sum);
    }

    // TODO is unwrap safe here?
    bytes_to_host(&host).unwrap()
}

#[cfg(test)]
mod tests {
    use std::net::IpAddr;
    use super::{bytes_to_host, extract_host_from_grpc_bytes, Session, ServiceId};

    #[test]
    fn test_bytes_to_host() {
        let a1 = [0; 4];
        let a2 = [0; 1];
        let a3 = [0; 5];
        let a4 = [0; 16];
        let a5 = [0; 26];

        let r1 = bytes_to_host(&a1);
        let r2 = bytes_to_host(&a2);
        let r3 = bytes_to_host(&a3);
        let r4 = bytes_to_host(&a4);
        let r5 = bytes_to_host(&a5);

        assert!(r1.is_ok());
        assert!(r2.is_err());
        assert!(r3.is_err());
        assert!(r4.is_ok());
        assert!(r5.is_err());
    }

    #[test]
    fn test_extract_host_from_grpc_bytes() {
        let a = b"127.127.127.127:65535";
        let b = extract_host_from_grpc_bytes(a);
        let ip: IpAddr = "127.127.127.127".parse().unwrap();
        assert_eq!(b, ip);

        let a = b"0.0.0.127:65535";
        let b = extract_host_from_grpc_bytes(a);
        let ip: IpAddr = "0.0.0.127".parse().unwrap();
        assert_eq!(b, ip);

        let a = b"1.1.1.1:1";
        let b = extract_host_from_grpc_bytes(a);
        let ip: IpAddr = "1.1.1.1".parse().unwrap();
        assert_eq!(b, ip);
    }

    #[test]
    fn test_session() {
        let ip = [0; 4];
        let mut s = Session::new(ServiceId(1), ip);

        s.step_heartbeat_port();
        s.step_service_port();
        assert_eq!(s.service_port, 20_000 + 1);
        assert_eq!(s.heartbeat_port, 25_000 + 1);
        s.step_both();
        assert_eq!(s.service_port, 20_000 + 2);
        assert_eq!(s.heartbeat_port, 25_000 + 2);
    }
}
