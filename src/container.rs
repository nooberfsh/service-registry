use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;
use std::net::SocketAddr;

use grpcio::{ChannelBuilder, EnvBuilder, Environment};
use grpcio::Error;
use protobuf::{Message as ProtoMessage, MessageStatic};
use mio::{Poll, Registration, SetReadiness, Ready, Token, PollOpt, Events};

use heartbeat::Server as HeartbeatServer;
use super::registry_proto_grpc::*;
use super::registry_proto::*;
use super::ServiceId;

const SHUTDOWN_TOKEN: Token = Token(0);
const HEARTBEAT_TOKEN: Token = Token(1);

#[derive(Clone, Copy)]
struct Meta {
    service_port: Option<u16>,
    heartbeat_port: Option<u16>,
}

impl Meta {
    fn new() -> Self {
        Meta {
            service_port: None,
            heartbeat_port: None,
        }
    }

    fn clean(&mut self) {
        self.service_port = None;
        self.heartbeat_port = None;
    }

    fn set_service_port(&mut self, port: u16) {
        self.service_port = Some(port);
    }

    fn set_heartbeat_port(&mut self, port: u16) {
        self.heartbeat_port = Some(port);
    }

    fn has_both_port(&self) -> bool {
        self.service_port.is_some() && self.heartbeat_port.is_some()
    }

    fn has_service_port(&self) -> bool {
        self.service_port.is_some()
    }

    fn has_heartbeat_port(&self) -> bool {
        self.heartbeat_port.is_some()
    }
}

pub struct Container<P, Q, E: Executor>
where
    P: MessageStatic,
    Q: ProtoMessage,
    E: Executor,
{
    heartbeat_server: HeartbeatServer<P, Q>,
    rpc_env: Arc<Environment>,
    config: Config,
    meta: Meta,
    executor: E,

    thread_handle: Option<JoinHandle<()>>,

    // used in drop
    shutdown_registration: Registration,
    shutdown_set_readiness: SetReadiness,

    // used in heartbeat notify
    heartbeat_registration: Registration,
    heartbeat_set_readiness: SetReadiness,
}

struct Inner {
    heartbeat_set_readiness: SetReadiness,
    env: Arc<Environment>,
    service_port: u16,
    heartbeat_port: u16,
    service_id: ServiceId,
    config: Config,
}

impl Inner {
    fn re_register(&self) -> Result<ReRegisterResponse, RpcError> {
        let mut req = ReRegisterRequest::new();
        req.heartbeat_port = u32::from(self.heartbeat_port);
        req.service_port = u32::from(self.service_port);
        req.service_id = self.service_id.0;

        let addr = format!("{}", self.config.server_addr);
        let ch = ChannelBuilder::new(Arc::clone(&self.env)).connect(&addr);
        let client = RegisterClient::new(ch);
        client.re_register(req).map_err(From::from)
    }
}

#[derive(Debug)]
pub enum RpcError {
    RpcErr(Error),
    ServerCrashed,
}

impl From<Error> for RpcError {
    fn from(e: Error) -> Self {
        RpcError::RpcErr(e)
    }
}

#[derive(Clone, Debug)]
pub struct Config {
    pub heartbeat_interval: Duration,
    pub server_addr: SocketAddr,
}

impl Config {
    pub fn new(heartbeat_interval: Duration, server_addr: SocketAddr) -> Self {
        Config {
            heartbeat_interval: heartbeat_interval,
            server_addr: server_addr,
        }
    }
}

pub trait Executor {
    fn service_id(&self) -> ServiceId;
    fn run(&mut self, port: u16) -> bool;
    fn stop(&mut self) {}
}

impl<P, Q, E> Container<P, Q, E>
where
    P: MessageStatic,
    Q: ProtoMessage,
    E: Executor,
{
    pub fn new<F>(config: Config, gen_rsp: F, executor: E) -> Self
    where
        F: Fn(P) -> Q + Send + Sync + 'static,
    {
        let (shutdown_registration, shutdown_set_readiness) = Registration::new2();
        let (heartbeat_registration, heartbeat_set_readiness) = Registration::new2();
        let tmp = heartbeat_set_readiness.clone();
        let f = move |req| {
            tmp.set_readiness(Ready::readable()).unwrap();
            gen_rsp(req)
        };
        Container {
            heartbeat_server: HeartbeatServer::new("heartbeat_server", f),
            rpc_env: Arc::new(EnvBuilder::new().build()),
            thread_handle: None,
            config: config,
            meta: Meta::new(),
            executor: executor,

            shutdown_registration: shutdown_registration,
            shutdown_set_readiness: shutdown_set_readiness,

            heartbeat_registration: heartbeat_registration,
            heartbeat_set_readiness: heartbeat_set_readiness,
        }
    }

    fn register_service(&self) -> Result<RegisterResponse, RpcError> {
        let addr = format!("{}", self.config.server_addr);
        let ch = ChannelBuilder::new(Arc::clone(&self.rpc_env)).connect(&addr);
        let client = RegisterClient::new(ch);
        let mut req = RegisterRequest::new();
        req.set_service_id(self.executor.service_id().0);
        client.register(req).map_err(|e| e.into())
    }

    fn report_status(&self, session_id: u64) -> Result<StatusResponse, RpcError> {
        let mut req = StatusRequest::new();
        req.service_succeed = self.meta.has_service_port();
        req.heartbeat_succeed = self.meta.has_heartbeat_port();
        req.session_id = session_id;

        let addr = format!("{}", self.config.server_addr);
        let ch = ChannelBuilder::new(Arc::clone(&self.rpc_env)).connect(&addr);
        let client = RegisterClient::new(ch);
        client.report_status(req).map_err(|e| e.into())
    }

    fn register_registration(&mut self, poll: &Poll) {
        poll.register(
            &self.shutdown_registration,
            SHUTDOWN_TOKEN,
            Ready::readable(),
            PollOpt::edge(),
        ).unwrap();
        poll.register(
            &self.heartbeat_registration,
            HEARTBEAT_TOKEN,
            Ready::readable(),
            PollOpt::edge(),
        ).unwrap();
    }

    fn register_and_run(&mut self) -> Result<(), RpcError> {
        let rsp = self.register_service()?;
        let mut service_port = rsp.service_port as u16;
        let mut heartbeat_port = rsp.heartbeat_port as u16;

        while !self.meta.has_both_port() {
            if !self.meta.has_service_port() && self.executor.run(service_port) {
                self.meta.set_service_port(service_port);
            }

            if !self.meta.has_heartbeat_port() &&
                self.heartbeat_server.start(heartbeat_port).is_ok()
            {
                self.meta.set_heartbeat_port(heartbeat_port);
            }

            let status_rsp = self.report_status(rsp.session_id)
                .and_then(|rsp| {
                    if rsp.succeed {
                        Ok(rsp)
                    } else {
                        //indicate server crashed before we report our status,
                        Err(RpcError::ServerCrashed)
                    }
                })
                .map_err(|e| {
                    //reset state;
                    if self.meta.has_service_port() {
                        self.executor.stop();
                    }
                    self.meta.clean();
                    e
                })?;

            service_port = status_rsp.service_port as u16;
            heartbeat_port = status_rsp.heartbeat_port as u16;
        }
        Ok(())
    }

    fn inner(&mut self) -> Inner {
        Inner {
            heartbeat_set_readiness: self.heartbeat_set_readiness.clone(),
            env: Arc::clone(&self.rpc_env),

            service_port: self.meta.service_port.unwrap(),
            heartbeat_port: self.meta.heartbeat_port.unwrap(),
            config: self.config.clone(),
            service_id: self.executor.service_id(),
        }
    }

    pub fn start(&mut self) -> Result<(), RpcError> {
        self.register_and_run()?;

        let poll = Poll::new().unwrap();
        self.register_registration(&poll);
        let inner = self.inner();

        let handle = thread::Builder::new()
            .name("container".to_string())
            .spawn(move || Self::begin_loop(poll, inner))
            .unwrap();
        self.thread_handle = Some(handle);
        Ok(())
    }

    fn begin_loop(poll: Poll, inner: Inner) {
        let mut events = Events::with_capacity(4);
        loop {
            let num = poll.poll(&mut events, Some(inner.config.heartbeat_interval))
                .unwrap();
            if num == 0 {
                //indicate registry server did not touch us for heartbeat_interval time
                warn!("lost connection to server, begin to re_register");
                match inner.re_register() {
                    Ok(rsp) => {
                        if rsp.succeed {
                            info!("re_register succeed");
                        } else {
                            warn!("re_register failed reason: {:?}", rsp.msg);
                            thread::sleep(Duration::from_secs(1));
                        }
                    }
                    Err(e) => {
                        warn!("re_register failed reason: {:?}", e);
                        thread::sleep(Duration::from_secs(1));
                    }
                }
            }
            for event in &events {
                if event.token() == SHUTDOWN_TOKEN && event.readiness().is_readable() {
                    info!("receive stop signal");
                    return;
                } else if event.token() == HEARTBEAT_TOKEN && event.readiness().is_readable() {
                    trace!("receive heartbeat");
                    inner
                        .heartbeat_set_readiness
                        .set_readiness(Ready::empty())
                        .unwrap();
                }
            }
        }
    }
}

impl<P, Q, E> Drop for Container<P, Q, E>
where
    P: MessageStatic,
    Q: ProtoMessage,
    E: Executor,
{
    fn drop(&mut self) {
        self.shutdown_set_readiness
            .set_readiness(Ready::readable())
            .unwrap();
        if let Some(h) = self.thread_handle.take() {
            self.executor.stop();
            h.join().unwrap();
        }
    }
}
