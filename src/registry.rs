use std::thread::{self, JoinHandle};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{self, Sender, Receiver};

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

    pub fn get_all_services(&self) -> Vec<Service> {
        let lock = self.services.lock().unwrap();
        lock.values().map(|sd| sd.service.clone()).collect()
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
