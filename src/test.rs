#[cfg(test)]
mod tests {
    extern crate bytes;
    extern crate env_logger;

    use std::sync::{Arc, Mutex};
    use std::sync::atomic::{AtomicUsize, AtomicBool, Ordering};
    use std::net::{TcpListener, TcpStream, SocketAddr};
    use std::io::{Read, Write};
    use std::thread;
    use std::time::Duration;

    use grpcio::{Environment, RpcContext, ServerBuilder, UnarySink, Server as GrpcServer};
    use protobuf::core::parse_from_bytes;
    use protobuf::Message;
    use protobuf::stream::CodedOutputStream;
    use bytes::{BigEndian, ByteOrder};
    use futures::Future;

    use heartbeat::{self, HeartbeatRequest, HeartbeatResponse};
    use super::client::{Client, Config as ClientConfig};
    use super::server::{Server, Config as ServerConfig};
    use super::register_proto_grpc::*;
    use super::register_proto::*;
    use super::ServiceId;

    const HELLO_WORLD_SERVICE_ID: ServiceId = ServiceId(0);
    const SWIM_SERVICE_ID: ServiceId = ServiceId(1);
    const RUN_SERVICE_ID: ServiceId = ServiceId(2);
    const SING__SERVICE_ID: ServiceId = ServiceId(3);

    #[derive(Clone)]
    struct RegisterService {
        heartbeat_port: Arc<AtomicUsize>,
        service_port: Arc<AtomicUsize>,
        session_id: Arc<AtomicUsize>,
        resume_count: Arc<AtomicUsize>,
        resume_enabled: Arc<AtomicBool>,
    }

    impl RegisterService {
        fn new() -> Self {
            RegisterService {
                service_port: Arc::new(AtomicUsize::new(21000)),
                heartbeat_port: Arc::new(AtomicUsize::new(25000)),
                session_id: Arc::new(AtomicUsize::new(10000)),
                resume_count: Arc::new(AtomicUsize::new(0)),
                resume_enabled: Arc::new(AtomicBool::new(true)),
            }
        }

        fn step_heartbeat(&self) {
            self.heartbeat_port.fetch_add(1, Ordering::SeqCst);
        }

        fn step_service(&self) {
            self.service_port.fetch_add(1, Ordering::SeqCst);
        }

        fn step_resume(&self) {
            self.resume_count.fetch_add(1, Ordering::SeqCst);
        }

        fn heartbeat_port(&self) -> u16 {
            self.heartbeat_port.load(Ordering::SeqCst) as u16
        }

        fn session_id(&self) -> u64 {
            self.session_id.load(Ordering::SeqCst) as u64
        }

        fn service_port(&self) -> u16 {
            self.service_port.load(Ordering::SeqCst) as u16
        }

        fn resume_count(&self) -> usize {
            self.resume_count.load(Ordering::SeqCst)
        }

        fn reset_resume_count(&self) {
            self.resume_count.store(0, Ordering::SeqCst);
        }

        fn refuse_resume(&self) {
            self.resume_enabled.store(false, Ordering::SeqCst);
        }

        fn resume_enabled(&self) -> bool {
            self.resume_enabled.load(Ordering::SeqCst)
        }

        fn step_both(&self) {
            self.step_heartbeat();
            self.step_service();
        }

        fn gen_rsp(&self) -> RegisterResponse {
            let mut rsp = RegisterResponse::new();
            rsp.set_heartbeat_port(self.heartbeat_port() as u32);
            rsp.set_service_port(self.service_port() as u32);
            rsp.set_session_id(self.session_id());
            rsp
        }
    }

    impl Register for RegisterService {
        fn register(&self, ctx: RpcContext, _: RegisterRequest, sink: UnarySink<RegisterResponse>) {
            let rsp = self.gen_rsp();
            let f = sink.success(rsp).map_err(|e| warn!("{:?}", e));
            ctx.spawn(f);
        }

        fn resume(&self, ctx: RpcContext, _: ResumeRequest, sink: UnarySink<ResumeResponse>) {
            self.step_resume();
            let mut rsp = ResumeResponse::new();
            if self.resume_enabled() {
                rsp.set_succeed(true);
                rsp.set_msg("succeed".to_string());
            } else {
                rsp.set_succeed(false);
                rsp.set_msg("refuse resume".to_string());
            }
            let f = sink.success(rsp).map_err(|e| warn!("{:?}", e));
            ctx.spawn(f);
        }

        fn report_status(
            &self,
            ctx: RpcContext,
            req: StatusRequest,
            sink: UnarySink<RegisterResponse>,
        ) {
            if !req.heartbeat_succeed && !req.service_succeed {
                self.step_both();
            } else if !req.heartbeat_succeed {
                self.step_heartbeat();
            } else if !req.service_succeed {
                self.step_service();
            } else {
            };

            let rsp = self.gen_rsp();
            let f = sink.success(rsp).map_err(|e| warn!("{:?}", e));
            ctx.spawn(f);
        }
    }

    fn gen_server(s: RegisterService) -> GrpcServer {
        let env = Arc::new(Environment::new(1));
        let service = create_register(s);
        let mut server = ServerBuilder::new(env)
            .register_service(service)
            .bind("0.0.0.0", 20000)
            .build()
            .unwrap();
        server.start();
        server
    }

    fn send_req(port: u16) {
        let addr_str = "127.0.0.1:".to_string() + &(format!("{}", port));
        let addr: SocketAddr = addr_str.parse().unwrap();
        let mut socket = TcpStream::connect(addr.clone()).unwrap();

        let request = heartbeat::default_hearbeat_request();
        let size = request.compute_size();
        // 4 bytes for tokio_io frame header;
        let mut v = vec![0; (4 + size) as usize];
        // tokio_io frame use big endian;
        BigEndian::write_u32(&mut v[0..4], size);
        {
            let mut output = CodedOutputStream::bytes(&mut v[4..]);
            request.write_to(&mut output).unwrap(); // TODO error handle;
        }

        socket.write_all(&v).unwrap();
        let mut v2 = vec![];
        socket.read_to_end(&mut v2).unwrap();

        let response: HeartbeatResponse = parse_from_bytes(&v2[4..]).unwrap();
        assert_eq!(response, heartbeat::default_hearbeat_response());
    }

    #[test]
    fn test_register_client() {
        let _ = env_logger::init();

        let gen_rsp = |_: HeartbeatRequest| heartbeat::default_hearbeat_response();
        let count = Arc::new(AtomicUsize::new(0));
        let tmp = count.clone();
        let run_service = move |port| {
            let ports = [21000, 21001, 21002];
            if ports.contains(&port) {
                tmp.fetch_add(1, Ordering::SeqCst);
                return false;
            }
            info!("run service with port: {}", port);
            true
        };
        let config = ClientConfig::new(
            HELLO_WORLD_SERVICE_ID,
            Duration::from_millis(200),
            "127.0.0.1:20000".parse().unwrap(),
        );
        let mut client = Client::new("test-register-client", config, gen_rsp, run_service);
        let _ = client.init().unwrap_err();
        let service = RegisterService::new();
        let _server = gen_server(service.clone());

        {
            let _lisener1 = TcpListener::bind("0.0.0.0:25000");
            let _lisener2 = TcpListener::bind("0.0.0.0:25001");
            let _lisener3 = TcpListener::bind("0.0.0.0:25002");
            client.init().unwrap();

            assert_eq!(count.load(Ordering::SeqCst), 3);
            assert_eq!(service.service_port(), 21003);
            assert_eq!(service.heartbeat_port(), 25003);
        }

        client.start();

        for _ in 0..5 {
            thread::sleep(Duration::from_millis(100));
            send_req(25003);
        }

        assert_eq!(service.resume_count(), 0);

        thread::sleep(Duration::from_millis(1100));
        assert_eq!(service.resume_count(), 5);
        for _ in 0..2 {
            send_req(25003);
            thread::sleep(Duration::from_millis(100));
        }
        send_req(25003);
        assert_eq!(service.resume_count(), 5);

        service.refuse_resume();
        service.reset_resume_count();
        thread::sleep(Duration::from_millis(5100));
        assert_eq!(service.resume_count(), 5);
    }

    fn new_client(name: &str, id: ServiceId) -> Client<HeartbeatRequest, HeartbeatResponse> {
        lazy_static! {
            static ref USED_PORTS: Arc<Mutex<Vec<u16>>> = {
                Arc::new(Mutex::new(vec![]))
            };
        }
        let gen_rsp = |_: HeartbeatRequest| heartbeat::default_hearbeat_response();
        let run_service = |port| {
            let mut lock = USED_PORTS.lock().unwrap();
            if lock.contains(&port) {
                false
            } else {
                lock.push(port);
                true
            }
        };
        let config = ClientConfig::new(
            id,
            Duration::from_millis(200),
            "127.0.0.1:40000".parse().unwrap(),
        );
        Client::new(name, config, gen_rsp, run_service)
    }

    #[test]
    fn test_register_server() {
        let _ = env_logger::init();

        let server_config = ServerConfig {
            server_port: 40000,
            service_port_base: 40001,
            heartbeat_port_base: 42000,
            heartbeat_interval: Duration::from_millis(100),
        };
        let mut server = Server::<HeartbeatRequest, HeartbeatResponse>::new(
            "test-register-server",
            server_config.clone(),
        );
        server.start(heartbeat::default_hearbeat_request());

        let mut swim_client = new_client("swim", SWIM_SERVICE_ID);

        swim_client.init().unwrap();
        thread::sleep(Duration::from_millis(5));
        let s = server.borrow(SWIM_SERVICE_ID);
        assert!(s.is_some());
        server.giveback(SWIM_SERVICE_ID, s.unwrap());
        let s = server.borrow(SWIM_SERVICE_ID);
        assert!(s.is_some());
        server.giveback(SWIM_SERVICE_ID, s.unwrap());

        let mut run_client1 = new_client("run1", RUN_SERVICE_ID);
        let mut run_client2 = new_client("run2", RUN_SERVICE_ID);
        let mut sing_client = new_client("sing", SING__SERVICE_ID);
        run_client1.init().unwrap();
        run_client1.start();
        run_client2.init().unwrap();
        run_client2.start();
        sing_client.init().unwrap();
        sing_client.start();
        thread::sleep(Duration::from_millis(5));
        let s1 = server.borrow(RUN_SERVICE_ID);
        let s2 = server.borrow(RUN_SERVICE_ID);
        assert!(s1.is_some());
        assert!(s2.is_some());
        let s3 = server.borrow(SING__SERVICE_ID);
        assert!(s3.is_some());

        info!("{:?}", server.get_all_service());
        drop(swim_client);
        thread::sleep(Duration::from_millis(105));
        info!("borrow swim serverice");
        let s4 = server.borrow(SWIM_SERVICE_ID);
        assert!(s4.is_none());
        let s5 = server.borrow(SING__SERVICE_ID);
        assert!(s5.is_none());

        drop(server);
        let mut server = Server::<HeartbeatRequest, HeartbeatResponse>::new(
            "test-register-server",
            server_config.clone(),
        );
        server.start(heartbeat::default_hearbeat_request());

        thread::sleep(Duration::from_millis(1500));
        let s1 = server.borrow(RUN_SERVICE_ID);
        let s2 = server.borrow(RUN_SERVICE_ID);
        assert!(s1.is_some());
        assert!(s2.is_some());
        let s3 = server.borrow(SING__SERVICE_ID);
        assert!(s3.is_some());
    }
}
