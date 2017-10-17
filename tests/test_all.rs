extern crate service_registry;

extern crate worker;
extern crate protobuf;

mod util;

use std::thread;
use std::time::{Duration, Instant};
use std::sync::mpsc::{self, RecvTimeoutError};

use protobuf::Message;

use service_registry::heartbeat::{Hub, Target, HubBuilder, TargetBuilder, Server};
use service_registry::heartbeat::heartbeat_proto::*;

use self::util::{simple_heartbeat_request, simple_heartbeat_response, create_server};

#[test]
fn test_all() {}
