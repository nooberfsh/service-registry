use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;
use std::net::SocketAddr;

use grpcio::{ChannelBuilder, EnvBuilder, Environment};
use grpcio::Error;
use protobuf::{Message as ProtoMessage, MessageStatic};
use mio::{Poll, Registration, SetReadiness, Ready, Token, PollOpt, Events};

use heartbeat::Server;
use super::register_proto_grpc::*;
use super::register_proto::*;
use super::ServiceId;

type RunService = Box<Fn(u16) -> bool + Send + Sync + 'static>;

const STOP_TOKEN: Token = Token(0);
const HEARTBEAT_TOKEN: Token = Token(1);

pub struct Client<P, Q> {
    name: String,
    heartbeat_server: Server<P, Q>,
    run_service: RunService,
    rpc_env: Arc<Environment>,
    handle: Option<JoinHandle<()>>,
    service_port: Option<u16>,
    heartbeat_port: Option<u16>,
    config: Config,
    service_succeed: bool,
    heartbeat_succeed: bool,

    // used in drop
    registration: Registration,
    set_readiness: SetReadiness,

    // used in heartbeat_server
    registration2: Registration,
    set_readiness2: SetReadiness,
}

struct Inner {
    heartbeat_readiness: SetReadiness,
    env: Arc<Environment>,
    register_interval: Duration,
    service_port: u16,
    heartbeat_port: u16,
    config: Config,
}

impl Inner {
    fn gen_reconnect(&self) -> Reconnect {
        Reconnect {
            service_port: self.service_port,
            heartbeat_port: self.heartbeat_port,
            service_id: self.config.service_id,
            server_addr: self.config.server_addr,
        }
    }
}

#[derive(Debug)]
pub struct ConnectFailed(Error);

impl From<Error> for ConnectFailed {
    fn from(e: Error) -> Self {
        ConnectFailed(e)
    }
}

#[derive(Clone, Debug)]
pub struct Config {
    pub service_id: ServiceId,
    pub register_interval: Duration,
    pub server_addr: SocketAddr,
}

impl Config {
    pub fn new(id: ServiceId, du: Duration, addr: SocketAddr) -> Self {
        Config {
            service_id: id,
            register_interval: du,
            server_addr: addr,
        }
    }
}

impl<P, Q> Client<P, Q>
where
    P: MessageStatic + ProtoMessage,
    Q: MessageStatic + ProtoMessage,
{
    pub fn new<N, F1, F2>(name: N, config: Config, gen_rsp: F1, run_service: F2) -> Self
    where
        N: Into<String>,
        F1: Fn(P) -> Q + Send + Sync + 'static,
        F2: Fn(u16) -> bool + Send + Sync + 'static,
    {
        let (r1, s1) = Registration::new2();
        let (r2, s2) = Registration::new2();
        let tmp = s2.clone();
        let f = move |req| {
            tmp.set_readiness(Ready::readable()).unwrap();
            gen_rsp(req)
        };
        Client {
            name: name.into(),
            heartbeat_server: Server::new("heartbeat_server", f),
            run_service: Box::new(run_service),
            rpc_env: Arc::new(EnvBuilder::new().build()),
            handle: None,
            service_port: None,
            heartbeat_port: None,
            config: config,
            service_succeed: false,
            heartbeat_succeed: false,

            registration: r1,
            set_readiness: s1,
            registration2: r2,
            set_readiness2: s2,
        }
    }

    fn register_service(&self) -> Result<RegisterResponse, Error> {
        let ch = ChannelBuilder::new(self.rpc_env.clone()).connect(&format!(
            "{}",
            self.config
                .server_addr
        ));
        let client = RegisterClient::new(ch);
        let req = self.new_register_request();
        client.register(req)
    }

    fn report_status(&self, st: Status) -> Result<RegisterResponse, Error> {
        let ch = ChannelBuilder::new(self.rpc_env.clone()).connect(&format!(
            "{}",
            self.config
                .server_addr
        ));
        let client = RegisterClient::new(ch);
        let req = st.into();
        client.report_status(req)
    }

    fn new_register_request(&self) -> RegisterRequest {
        let mut req = RegisterRequest::new();
        req.set_service_id(self.config.service_id.0);
        req
    }

    fn register_registration(&mut self, poll: &Poll) {
        poll.register(
            &self.registration,
            STOP_TOKEN,
            Ready::readable(),
            PollOpt::edge(),
        ).unwrap();
        poll.register(
            &self.registration2,
            HEARTBEAT_TOKEN,
            Ready::readable(),
            PollOpt::edge(),
        ).unwrap();
    }

    pub fn init(&mut self) -> Result<(), ConnectFailed> {
        let rsp = self.register_service()?;

        let mut service_port = rsp.service_port;
        let mut heartbeat_port = rsp.heartbeat_port;
        while !self.service_succeed || !self.heartbeat_succeed {
            if !self.service_succeed {
                self.service_succeed = (self.run_service)(service_port as u16);
            }
            if !self.heartbeat_succeed {
                self.heartbeat_succeed = self.heartbeat_server.start(heartbeat_port as u16).is_ok();
            }

            let st = Status {
                service_succeed: self.service_succeed,
                heartbeat_succeed: self.heartbeat_succeed,
                session_id: rsp.session_id,
            };
            let statusrsp = self.report_status(st)?;

            if !self.service_succeed {
                service_port = statusrsp.service_port;
            }
            if !self.heartbeat_succeed {
                heartbeat_port = statusrsp.heartbeat_port;
            }
        }
        self.service_port = Some(service_port as u16);
        self.heartbeat_port = Some(heartbeat_port as u16);
        Ok(())
    }

    fn inner(&mut self) -> Inner {
        Inner {
            heartbeat_readiness: self.set_readiness2.clone(),
            env: self.rpc_env.clone(),
            register_interval: self.config.register_interval,
            service_port: self.service_port.unwrap(),
            heartbeat_port: self.heartbeat_port.unwrap(),
            config: self.config.clone(),
        }
    }

    pub fn start(&mut self) {
        let poll = Poll::new().unwrap();
        self.register_registration(&poll);
        let inner = self.inner();

        let handle = thread::Builder::new()
            .name(self.name.clone())
            .spawn(move || Self::begin_loop(poll, inner))
            .unwrap();
        self.handle = Some(handle);
    }

    fn begin_loop(poll: Poll, inner: Inner) {
        let mut events = Events::with_capacity(4);
        loop {
            let num = poll.poll(&mut events, Some(inner.register_interval))
                .unwrap();
            if num == 0 {
                //indicate service register server did not touch us for register_interval time
                warn!("lost connection to server, begin to reconnect");
                match reconnect(inner.env.clone(), inner.gen_reconnect()) {
                    Ok(rsp) => {
                        if rsp.succeed {
                            info!("reconnect succeed");
                        } else {
                            warn!("reconnect failed reason: {:?}", rsp.msg);
                            thread::sleep(Duration::from_secs(1));
                        }
                    }
                    Err(e) => {
                        warn!("reconnect failed reason: {:?}", e);
                        thread::sleep(Duration::from_secs(1));
                    }
                }
            }
            for event in &events {
                if event.token() == STOP_TOKEN && event.readiness().is_readable() {
                    info!("receive stop signal");
                    return;
                } else if event.token() == HEARTBEAT_TOKEN && event.readiness().is_readable() {
                    trace!("receive heartbeat");
                    inner
                        .heartbeat_readiness
                        .set_readiness(Ready::empty())
                        .unwrap();
                }
            }
        }

    }
}

#[derive(Copy, Clone, Debug)]
struct Reconnect {
    service_port: u16,
    heartbeat_port: u16,
    service_id: ServiceId,
    server_addr: SocketAddr,
}

impl From<Reconnect> for ResumeRequest {
    fn from(re: Reconnect) -> Self {
        let mut req = ResumeRequest::new();
        req.set_service_id(re.service_id.0);
        req.set_heartbeat_port(u32::from(re.heartbeat_port));
        req.set_service_port(u32::from(re.service_port));
        req
    }
}

#[derive(Copy, Clone, Debug)]
struct Status {
    service_succeed: bool,
    heartbeat_succeed: bool,
    session_id: u64,
}

impl From<Status> for StatusRequest {
    fn from(s: Status) -> Self {
        let mut req = StatusRequest::new();
        req.set_service_succeed(s.service_succeed);
        req.set_heartbeat_succeed(s.heartbeat_succeed);
        req.set_session_id(s.session_id);
        req
    }
}

fn reconnect(env: Arc<Environment>, reconnect: Reconnect) -> Result<ResumeResponse, Error> {
    let addr = format!("{}", reconnect.server_addr);
    let ch = ChannelBuilder::new(env).connect(&addr);
    let client = RegisterClient::new(ch);
    let req = reconnect.into();
    client.resume(req)
}

impl<P, Q> Drop for Client<P, Q> {
    fn drop(&mut self) {
        self.set_readiness.set_readiness(Ready::readable()).unwrap();
        self.handle.take().map(|h| h.join().unwrap());
    }
}
