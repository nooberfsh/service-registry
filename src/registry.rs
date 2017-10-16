use std::thread::{self, JoinHandle};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{self, Sender, Receiver};
use std::time::Duration;

use protobuf::{Message as ProtoMessage, MessageStatic};
use grpcio::{Error as GrpcError, Server as GrpcServer};
use uuid::Uuid;

use heartbeat::{Hub, HubHandle, TargetBuilder, Error as HeartbeatError};
use super::{Service, rpc_server};

#[derive(PartialEq, Eq, Clone, Debug)]
struct ServiceDetail {
    service: Service,
    uuid: Uuid,
}

impl ServiceDetail {
    fn new(service: Service, uuid: Uuid) -> Self {
        ServiceDetail {
            service: service,
            uuid: uuid,
        }
    }
}

type ServiceDetails = Arc<Mutex<HashMap<Uuid, ServiceDetail>>>;

pub struct Registry<P, Q>
where
    P: ProtoMessage,
    Q: MessageStatic,
{
    services: ServiceDetails,
    sender: Sender<Message<Q>>,
    grpc_server: Option<GrpcServer>,
    hub: Option<Hub<P, Q>>,
    thread_handle: Option<JoinHandle<()>>,
}

struct Inner<P, Q>
where
    P: ProtoMessage,
    Q: MessageStatic,
{
    services: ServiceDetails,
    sender: Sender<Message<Q>>,
    receiver: Receiver<Message<Q>>,
    heartbeat_interval: Duration,
    heartbeat_timeout: Duration,
    hub_handle: HubHandle<P, Q>,
    service_available_handle: Box<Fn(Service) + Send + 'static>,
    service_droped_handle: Box<Fn(Service) + Send + 'static>,
}

impl<P, Q> Registry<P, Q>
where
    P: ProtoMessage,
    Q: MessageStatic,
{
    pub fn new<F1, F2>(
        server_port: u16,
        heartbeat_interval: Duration,
        heartbeat_timeout: Duration,
        hub: Hub<P, Q>,
        service_available_handle: F1,
        service_droped_handle: F2,
    ) -> Result<Self, GrpcError>
    where
        F1: Fn(Service) + Send + 'static,
        F2: Fn(Service) + Send + 'static,
    {
        let (tx, rx) = mpsc::channel();
        // create grpc server;
        // grpc was droped before the loop routine, so it is safe to unwrap.
        let register_handle = {
            let sender = tx.clone();
            move |service| sender.send(Message::Register(service)).unwrap()
        };
        let re_register_handle = {
            let sender = tx.clone();
            move |service| sender.send(Message::ReRegister(service)).unwrap()
        };
        let mut grpc_server =
            rpc_server::create_grpc_server(server_port, register_handle, re_register_handle)?;
        grpc_server.start();

        let services = Default::default();
        let inner = Inner {
            services: Arc::clone(&services),
            sender: tx.clone(),
            receiver: rx,
            heartbeat_interval: heartbeat_interval,
            heartbeat_timeout: heartbeat_timeout,
            hub_handle: hub.get_handle(),
            service_available_handle: Box::new(service_available_handle),
            service_droped_handle: Box::new(service_droped_handle),
        };

        let thread_handle = thread::Builder::new()
            .name("registry_notifier".to_string())
            .spawn(move || Self::begin_loop(inner))
            .unwrap();

        Ok(Registry {
            services: services,
            sender: tx,
            grpc_server: Some(grpc_server),
            hub: Some(hub),
            thread_handle: Some(thread_handle),
        })
    }

    fn begin_loop(inner: Inner<P, Q>) {
        loop {
            match inner.receiver.recv().unwrap() {
                Message::Register(service) |
                Message::ReRegister(service) => Self::add_service(service, &inner),
                Message::Heartbeat(uuid, res) => {
                    if let Err(e) = res {
                        let mut lock = inner.services.lock().unwrap();
                        let detail = lock.remove(&uuid).unwrap();
                        warn!(
                            "heartbeat to service:{:?} failed, reason:{:?}, remove this service",
                            detail,
                            e
                        );
                    }
                }
                Message::Stop => break,
            }
        }
    }

    fn add_service(service: Service, inner: &Inner<P, Q>) {
        let mut lock = inner.services.lock().unwrap();
        if lock.values().any(|sd| sd.service == service) {
            warn!(
                "add service:{:?} failed, it had been in the service table",
                service
            );
        }
        let sender = inner.sender.clone();
        let f = move |uuid, res| {
            let msg = Message::Heartbeat(uuid, res);
            sender.send(msg).unwrap();
        };
        let target = TargetBuilder::new(&service.heartbeat_addr())
            .interval(inner.heartbeat_interval)
            .timeout(inner.heartbeat_timeout)
            .cb(f)
            .build()
            .unwrap();

        let uuid = {
            if let Ok(uuid) = inner.hub_handle.add_target(target) {
                uuid
            } else {
                info!("add target to hub failed because hub was destroyed");
                return;
            }
        };
        lock.insert(uuid, ServiceDetail::new(service.clone(), uuid));
        (inner.service_available_handle)(service);
    }
}

enum Message<Q> {
    Register(Service),
    ReRegister(Service),
    Heartbeat(Uuid, Result<Q, HeartbeatError>),
    Stop,
}

impl<P, Q> Drop for Registry<P, Q>
where
    P: ProtoMessage,
    Q: MessageStatic,
{
    fn drop(&mut self) {
        self.grpc_server.take().unwrap();
        self.hub.take().unwrap();
        self.sender.send(Message::Stop).unwrap();
        self.thread_handle.take().unwrap().join().unwrap();
    }
}

#[cfg(test)]
mod tests {
    /*extern crate env_logger;

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
    }*/
}
