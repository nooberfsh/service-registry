use std::io;
use std::thread::{self, JoinHandle};
use std::net::{IpAddr, SocketAddr};
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering, ATOMIC_USIZE_INIT};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{self, Sender, Receiver};
use std::time::Duration;
use std::marker::PhantomData;

use protobuf::{Message as ProtoMessage, MessageStatic};
use super::registry_proto_grpc::{create_register, Register};
use super::registry_proto::{RegisterResponse, RegisterRequest, StatusRequest, ResumeRequest,
                            ResumeResponse};
use futures::Future;
use uuid::Uuid;

use heartbeat::Hub as HbClient;
use super::ServiceId;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
struct Service {
    id: ServiceId,
    addr: SocketAddr,
    uuid: Uuid,
    borrowed: bool,
}

impl Service {
    fn new<T: Into<SocketAddr>>(id: ServiceId, addr: T, uuid: Uuid) -> Self {
        Service {
            id: id,
            addr: addr.into(),
            uuid: uuid,
            borrowed: false,
        }
    }
}

#[derive(Clone)]
pub struct Config {
    pub server_port: u16,
    pub service_port_base: u16,
    pub heartbeat_port_base: u16,
    pub heartbeat_interval: Duration,
}

pub struct Server<P, Q> {
    name: String,
    services: Arc<Mutex<HashMap<Uuid, Service>>>,
    sender: Sender<Message>,
    receiver: Option<Receiver<Message>>,
    server: Option<GrpcServer>,
    handle: Option<JoinHandle<()>>,
    config: Config,
    __request: PhantomData<P>,
    __response: PhantomData<Q>,

    register_service_cb: Option<Box<Fn(ServiceId, SocketAddr) + Send + 'static>>,
    service_droped_cb: Option<Box<Fn(ServiceId, SocketAddr) + Send + 'static>>,
}

struct Inner<P, Q> {
    receiver: Receiver<Message>,
    heartbeat_manager: HbClient<P, Q>,
    services: Arc<Mutex<HashMap<Uuid, Service>>>,

    register_service_cb: Box<Fn(ServiceId, SocketAddr) + Send + 'static>,
    service_droped_cb: Box<Fn(ServiceId, SocketAddr) + Send + 'static>,
}

#[derive(Eq, PartialEq, Debug)]
enum Message {
    RegisterService(Session),
    HeartbeatFailed(Uuid),
    Resume(Resume),
    Stop,
}

impl<P, Q> Server<P, Q>
where
    P: ProtoMessage,
    Q: MessageStatic + ProtoMessage,
{
    pub fn new<N, F1, F2>(
        name: N,
        config: Config,
        register_service_cb: F1,
        service_droped_cb: F2,
    ) -> Self
    where
        N: Into<String>,
        F1: Fn(ServiceId, SocketAddr) + Send + 'static,
        F2: Fn(ServiceId, SocketAddr) + Send + 'static,
    {
        let (tx, rx) = mpsc::channel();
        Server {
            name: name.into(),
            services: Default::default(),
            sender: tx,
            receiver: Some(rx),
            server: None,
            handle: None,
            config: config,
            __request: PhantomData,
            __response: PhantomData,

            register_service_cb: Some(Box::new(register_service_cb)),
            service_droped_cb: Some(Box::new(service_droped_cb)),
        }
    }

    fn inner(&mut self, client: HbClient<P, Q>) -> Inner<P, Q> {
        Inner {
            services: self.services.clone(),
            receiver: self.receiver.take().unwrap(),
            heartbeat_manager: client,
            register_service_cb: self.register_service_cb.take().unwrap(),
            service_droped_cb: self.service_droped_cb.take().unwrap(),
        }
    }

    /*fn create_hearbeat_manager(&self) -> HbClient<P, Q> {
        let sender = self.sender.clone();
        let f = move |uuid, rsp: io::Result<Q>| {
            match rsp {
                // now there is nothing todo here, but maybe will use later.
                Ok(_) => {}
                Err(e) => {
                    warn!("{:?} heartbeat failed, reason: {:?}", uuid, e);
                    let msg = Message::HeartbeatFailed(uuid);
                    // tx will be droped when server is droping.
                    let _ = sender.send(msg);
                }
            }
        };
        HbClient::new("heartbeat_manager", self.config.heartbeat_interval, f)
    }*/

    fn create_rpc_server(&self) -> GrpcServer {
        let env = Arc::new(Environment::new(1));
        let register_service = RegisterService::new(self.sender.clone(), self.config.clone());
        let service = create_register(register_service);
        ServerBuilder::new(env)
            .register_service(service)
            .bind("0.0.0.0", self.config.server_port)
            .build()
            .unwrap()
    }

    pub fn start(&mut self, request: P) {
        // start heartbeat module;
        let mut heartbeat_manager = HbClient::new();

        // start register rpc server;
        let mut server = self.create_rpc_server();
        server.start();
        self.server = Some(server);

        let inner = self.inner(heartbeat_manager);

        let handle = thread::Builder::new()
            .name(self.name.clone())
            .spawn(move || Self::begin_loop(inner))
            .unwrap();
        self.handle = Some(handle);
    }

    fn begin_loop(inner: Inner<P, Q>) {
        loop {
            match inner.receiver.recv().unwrap() {
                Message::RegisterService(session) => {
                    /*let uuid = inner.heartbeat_manager.add_service(
                        session.heartbeat_addr(),
                    );
                    let addr = session.service_addr();
                    let sid = session.service_id;
                    {
                        let service = Service::new(sid, addr, uuid);
                        let mut lock = inner.services.lock().unwrap();
                        lock.insert(uuid, service);
                    }
                    (inner.register_service_cb)(sid, addr);*/
                }
                Message::HeartbeatFailed(uuid) => {
                    warn!("heartbeat failed {:?}", uuid);
                    let service = {
                        let mut lock = inner.services.lock().unwrap();
                        lock.remove(&uuid).unwrap()
                    };
                    (inner.service_droped_cb)(service.id, service.addr);
                }
                Message::Resume(resume) => {
                    /*let mut lock = inner.services.lock().unwrap();
                    for (_, service) in lock.iter() {
                        if service.addr == resume.service_addr() {
                            return;
                        }
                    }
                    let uuid = inner.heartbeat_manager.add_service(resume.heartbeat_addr());
                    let service = Service::new(resume.service_id, resume.service_addr(), uuid);
                    lock.insert(uuid, service);
                    drop(lock);
                    (inner.register_service_cb)(resume.service_id, resume.service_addr());*/
                }
                Message::Stop => return,
            }
        }
    }
}

impl<P, Q> Drop for Server<P, Q> {
    fn drop(&mut self) {
        self.sender.send(Message::Stop).unwrap();
        self.handle.take().map(|h| h.join().unwrap());
    }
}


#[cfg(test)]
mod tests {
    extern crate env_logger;

    use std::sync::mpsc;

    use grpcio::ChannelBuilder;

    use register::register_proto_grpc::*;
    use super::*;

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
        assert_eq!(b, vec![127, 127, 127, 127]);

        let a = b"0.0.0.127:65535";
        let b = extract_host_from_grpc_bytes(a);
        assert_eq!(b, vec![0, 0, 0, 127]);

        let a = b"1.1.1.1:1";
        let b = extract_host_from_grpc_bytes(a);
        assert_eq!(b, vec![1, 1, 1, 1]);
    }

    #[test]
    fn test_session() {
        let a1 = [0; 4];
        let a2 = [0; 0];

        let config = Config {
            server_port: 9999,
            service_port_base: 21000,
            heartbeat_port_base: 25000,
            heartbeat_interval: Duration::from_millis(100),
        };

        let s1 = Session::from_bytes(ServiceId(1), &a1, &config);
        let s2 = Session::from_bytes(ServiceId(1), &a2, &config);
        assert!(s1.is_ok());
        assert!(s2.is_err());

        let mut s1 = s1.unwrap();
        s1.step_heartbeat_port();
        s1.step_service_port();
        assert_eq!(s1.heartbeat_port, config.heartbeat_port_base + 1);
        assert_eq!(s1.service_port, config.service_port_base + 1);

        let sa = s1.service_addr();
        let ha = s1.heartbeat_addr();
        let r_sa = "0.0.0.0:".to_string() + &format!("{}", s1.service_port);
        let r_ha = "0.0.0.0:".to_string() + &format!("{}", s1.heartbeat_port);
        assert_eq!(sa, r_sa.parse().unwrap());
        assert_eq!(ha, r_ha.parse().unwrap());
    }

    #[test]
    fn test_register_service() {
        let config = Config {
            server_port: 9999,
            service_port_base: 21000,
            heartbeat_port_base: 25000,
            heartbeat_interval: Duration::from_millis(100),
        };
        let (tx, rx) = mpsc::channel::<Message>();
        let register_service = RegisterService::new(tx, config.clone());

        let a = [0; 4];
        let s = Session::from_bytes(ServiceId(1), &a, &config).unwrap();

        register_service.add_session(s.clone());
        {
            let lock = register_service.sessions.lock().unwrap();
            let v = lock.get(&s.session_id).unwrap();
            assert_eq!(v, &s);
        }

        let rs = register_service.step_both(s.session_id).unwrap();
        assert_eq!(rs.heartbeat_port, s.heartbeat_port + 1);
        assert_eq!(rs.service_port, s.service_port + 1);

        let rs = register_service.step_service(s.session_id).unwrap();
        assert_eq!(rs.heartbeat_port, s.heartbeat_port + 1);
        assert_eq!(rs.service_port, s.service_port + 2);

        let rs = register_service.step_heartbeat(s.session_id).unwrap();
        assert_eq!(rs.heartbeat_port, s.heartbeat_port + 2);
        assert_eq!(rs.service_port, s.service_port + 2);

        let fake_id = (s.session_id.0 + 1).into();
        let rs1 = register_service.step_both(fake_id);
        let rs2 = register_service.step_service(fake_id);
        let rs3 = register_service.step_heartbeat(fake_id);
        assert!(rs1.is_none());
        assert!(rs2.is_none());
        assert!(rs3.is_none());

        let rs = register_service.finish_session(s.session_id).unwrap();
        assert_eq!(rs.service_id, s.service_id);
        assert_eq!(rs.session_id, s.session_id);
        {
            let lock = register_service.sessions.lock().unwrap();
            assert_eq!(lock.len(), 0);
        }
        let msg = rx.recv().unwrap();
        assert_eq!(msg, Message::RegisterService(rs));

        let rs1 = register_service.finish_session(fake_id);
        assert!(rs1.is_none());

        let req = ResumeRequest::new();
        let resume = Resume::new(&req, a);
        register_service.resume(resume.clone());
        let msg = rx.recv().unwrap();
        assert_eq!(msg, Message::Resume(resume));
    }

    #[test]
    fn test_register_service_grpc_service() {
        let _ = env_logger::init();

        let config = Config {
            server_port: 9999,
            service_port_base: 21000,
            heartbeat_port_base: 25000,
            heartbeat_interval: Duration::from_millis(100),
        };

        let (tx, rx) = mpsc::channel::<Message>();
        let register_service = RegisterService::new(tx, config.clone());

        let env = Arc::new(Environment::new(1));
        let service = create_register(register_service.clone());
        let mut server = ServerBuilder::new(env.clone())
            .register_service(service)
            .bind("0.0.0.0", 50001)
            .build()
            .unwrap();
        server.start();

        let ch = ChannelBuilder::new(env.clone()).connect("127.0.0.1:50001");
        let client = RegisterClient::new(ch);
        let mut req = RegisterRequest::new();
        req.set_service_id(100);
        let rsp = client.register(req).unwrap();

        assert_eq!(rsp.heartbeat_port, config.heartbeat_port_base as u32);
        assert_eq!(rsp.service_port, config.service_port_base as u32);

        {
            let lock = register_service.sessions.lock().unwrap();
            let s = lock.get(&rsp.session_id.into());
            assert!(s.is_some());
            let s = s.unwrap();
            assert_eq!(s.service_id, ServiceId(100));
            assert_eq!(s.heartbeat_port, config.heartbeat_port_base);
            assert_eq!(s.service_port, config.service_port_base);
        }

        let mut req = StatusRequest::new();
        req.set_service_succeed(true);
        req.set_heartbeat_succeed(true);
        req.set_session_id(rsp.session_id);
        let rsp = client.report_status(req).unwrap();

        {
            let lock = register_service.sessions.lock().unwrap();
            let s = lock.get(&rsp.session_id.into());
            assert!(s.is_none());
        }

        let msg = rx.recv().unwrap();
        match msg {
            Message::RegisterService(s) => {
                assert_eq!(s.service_id, ServiceId(100));
            }
            _ => panic!(""),
        }

        let mut req = ResumeRequest::new();
        req.set_service_id(1000);
        req.set_service_port(55000);
        req.set_heartbeat_port(45000);
        let _ = client.resume(req).unwrap();

        let msg = rx.recv().unwrap();
        match msg {
            Message::Resume(r) => {
                assert_eq!(r.service_port, 55000);
                assert_eq!(r.heartbeat_port, 45000);
                assert_eq!(r.service_id, ServiceId(1000));
            }
            _ => panic!(""),
        }

        let mut req = RegisterRequest::new();
        req.set_service_id(999);
        let rsp = client.register(req).unwrap();

        let sp = rsp.service_port;
        let hp = rsp.heartbeat_port;

        let mut req = StatusRequest::new();
        req.set_service_succeed(false);
        req.set_heartbeat_succeed(true);
        req.set_session_id(rsp.session_id);
        let rsp = client.report_status(req).unwrap();
        let sp2 = rsp.service_port;
        let hp2 = rsp.heartbeat_port;

        assert_eq!(hp, hp2);
        assert_eq!(sp2 - sp, 1);

        let mut req = StatusRequest::new();
        req.set_service_succeed(true);
        req.set_heartbeat_succeed(false);
        req.set_session_id(rsp.session_id);
        let rsp = client.report_status(req).unwrap();
        let sp3 = rsp.service_port;
        let hp3 = rsp.heartbeat_port;
        assert_eq!(sp2, sp3);
        assert_eq!(hp3 - hp2, 1);

        let mut req = StatusRequest::new();
        req.set_service_succeed(true);
        req.set_heartbeat_succeed(true);
        req.set_session_id(rsp.session_id);
        let rsp = client.report_status(req).unwrap();

        {
            let lock = register_service.sessions.lock().unwrap();
            let s = lock.get(&rsp.session_id.into());
            assert!(s.is_none());
        }

        let msg = rx.recv().unwrap();
        match msg {
            Message::RegisterService(s) => {
                assert_eq!(s.service_id, ServiceId(999));
            }
            _ => panic!(""),
        }
    }
}
