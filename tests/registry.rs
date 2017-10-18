extern crate service_registry;

mod util;

use std::sync::mpsc;
use std::net::SocketAddr;
use std::thread;
use std::time::Duration;

use service_registry::registry::Registry;
use service_registry::heartbeat::{Hub, HubBuilder};
use service_registry::ServiceId;
use service_registry::container::{Container, Executor};
use service_registry::heartbeat::heartbeat_proto::*;

struct Exe {
    sid: ServiceId,
}

impl Executor for Exe {
    fn service_id(&self) -> ServiceId {
        self.sid
    }

    fn run(&mut self, _: u16) -> bool {
        true
    }
}

fn create_simple_container(
    server_addr: SocketAddr,
    heartbeat_interval: Duration,
    sid: ServiceId,
) -> Container<HeartbeatRequest, HeartbeatResponse, Exe> {
    let gen_rsp = |_| util::simple_heartbeat_response();
    Container::new(server_addr, heartbeat_interval, gen_rsp, Exe { sid: sid })
}

#[test]
fn test_registry() {
    let port = 12_000;
    let addr = ("127.0.0.1:".to_string() + &format!("{}", port))
        .parse()
        .unwrap();

    let interval = Duration::from_millis(50);
    let timeout = Duration::from_millis(200);
    let hub = HubBuilder::<HeartbeatRequest, HeartbeatResponse>::new(
        util::simple_heartbeat_request(),
    ).timeout(timeout)
        .interval(interval)
        .build()
        .unwrap();

    let (a_tx, a_rx) = mpsc::channel();
    let service_available_handle = move |s| { a_tx.send(s).unwrap(); };
    let (d_tx, d_rx) = mpsc::channel();
    let service_drop_handle = move |s| { d_tx.send(s).unwrap(); };

    let registry = Registry::new(port, hub, service_available_handle, service_drop_handle).unwrap();


    //create containers
    let max_interval = Duration::from_millis(100);

    let create_and_start = |sid| {
        let mut container = create_simple_container(addr, max_interval, sid);
        container.start().unwrap();
        container
    };

    let sida = 10_u64.into();
    let container_a = create_and_start(sida);
    let sa = a_rx.recv().unwrap();
    assert_eq!(sida, sa.service_id());

    let sidb = 20_u64.into();
    let container_b = create_and_start(sidb);
    let sb = a_rx.recv().unwrap();
    assert_eq!(sidb, sb.service_id());

    let sidc = 30_u64.into();
    let container_c = create_and_start(sidc);
    let sc = a_rx.recv().unwrap();
    assert_eq!(sidc, sc.service_id());

    let sidd = 30_u64.into();
    let container_d = create_and_start(sidd);
    let sd = a_rx.recv().unwrap();
    assert_eq!(sidd, sd.service_id());

    let gen_all_ids = || {
        let services = registry.get_all_services();
        let mut ids = services
            .into_iter()
            .map(|s| s.service_id())
            .collect::<Vec<_>>();
        ids.sort();
        ids
    };
    let ids = gen_all_ids();
    assert_eq!(ids, vec![sida, sidb, sidc, sidd]);

    drop(container_a);
    let dsa = d_rx.recv().unwrap();
    assert_eq!(sida, dsa.service_id());
    let ids = gen_all_ids();
    assert_eq!(ids, vec![sidb, sidc, sidd]);

    drop(container_c);
    let dsc = d_rx.recv().unwrap();
    assert_eq!(sidc, dsc.service_id());
    let ids = gen_all_ids();
    assert_eq!(ids, vec![sidb, sidd]);


    let side = 50_u64.into();
    let container_e = create_and_start(side);
    let se = a_rx.recv().unwrap();
    assert_eq!(side, se.service_id());
    let ids = gen_all_ids();
    assert_eq!(ids, vec![sidb, sidd, side]);

    thread::sleep(Duration::from_secs(3));
    let ids = gen_all_ids();
    assert_eq!(ids, vec![sidb, sidd, side]);
}
