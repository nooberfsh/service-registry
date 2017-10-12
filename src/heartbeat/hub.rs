use std::io;
use std::fmt;
use std::net::SocketAddr;
use std::time::Duration;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::thread::{self, JoinHandle};
use std::sync::mpsc::{self, Sender, Receiver};
use std::marker::PhantomData;

use futures::{Future, Sink, Stream};
use futures::future::Either;
use tokio_core::reactor::{Handle, Timeout};
use tokio_core::net::TcpStream;
use tokio_io::codec::length_delimited::Framed;
use protobuf::core::parse_from_bytes;
use protobuf::{Message as ProtoMessage, MessageStatic};
use uuid::Uuid;
use future_worker::{Runner, FutureWorker, Scheduler};

use super::Error;

pub struct Target<P, Q> {
    addr: SocketAddr,
    uuid: Uuid,
    interval: Duration,
    timeout: Duration,
    payload: Vec<u8>,
    cb: Box<Fn(Uuid, Result<Q, Error>) + Send + 'static>,
    _marker: PhantomData<P>,
}

impl<P, Q> Target<P, Q>
where
    P: ProtoMessage,
    Q: MessageStatic,
{
    pub fn new<F>(
        addr: &SocketAddr,
        interval: Duration,
        timeout: Duration,
        request: P,
        cb: F,
    ) -> Result<Self, Error>
    where
        F: Fn(Uuid, Result<Q, Error>) + Send + 'static,
    {
        let payload = request
            .write_to_bytes()
            .map_err(|e| {
                let s = format!("{:?}", e);
                Error::SerializeFailed(s)
            })
            .and_then(|v| if v.is_empty() {
                Err(Error::ZeroPayload)
            } else {
                Ok(v)
            })?;
        Ok(Target {
            addr: *addr,
            uuid: Uuid::new_v4(),
            interval: interval,
            timeout: timeout,
            payload: payload,
            cb: Box::new(cb),
            _marker: PhantomData,
        })
    }

    fn gen_task(&self) -> HeartbeatTask {
        HeartbeatTask {
            addr: self.addr,
            uuid: self.uuid,
            delay: self.interval,
            timeout: self.timeout,
            payload: self.payload.clone(),
        }
    }
}

impl<P, Q> fmt::Debug for Target<P, Q> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "Target[uudi={}, addr={}]", self.uuid, self.addr)
    }
}

struct HeartbeatTask {
    addr: SocketAddr,
    uuid: Uuid,
    delay: Duration,
    timeout: Duration,
    payload: Vec<u8>,
}

struct HeartbeatRunner<Q> {
    sender: Sender<Message<Q>>,
}

impl<Q> HeartbeatRunner<Q>
where
    Q: MessageStatic,
{
    fn gen_heartbeat_future(
        &self,
        task: HeartbeatTask,
        handle: &Handle,
    ) -> impl Future<Item = Q, Error = Error> {
        let payload = task.payload.into();
        let base = TcpStream::connect(&task.addr, handle).and_then(move |stream| {
            let frame = Framed::new(stream);
            frame.send(payload).and_then(move |stream: Framed<_>| {
                stream.into_future().map_err(|(e, _)| e).and_then(
                    |(item, _)| {
                        item.ok_or_else(|| io::Error::new(io::ErrorKind::Other, "closed by server"))
                    },
                )
            })
        });

        let delay = Timeout::new(task.delay, handle).unwrap().then(|_| base);
        let parse = delay.and_then(|r| parse_from_bytes::<Q>(&r).map_err(From::from));
        let timeout = Timeout::new(task.timeout, handle).unwrap();

        timeout.select2(parse).then(|r| {
            match r {
                Ok(r) => {
                    match r {
                        Either::A(_) => Err(Error::Timeout),
                        Either::B((q, _)) => Ok(q),
                    }
                }
                Err(e) => {
                    match e {
                        Either::A(_) => unreachable!(), // poll of Timeout never return Err,
                        Either::B((e, _)) => Err(Error::IoErr(e)),
                    }
                }
            }
        })
    }
}

impl<Q> Runner<HeartbeatTask> for HeartbeatRunner<Q>
where
    Q: MessageStatic + 'static,
{
    fn run(&mut self, task: HeartbeatTask, handle: &Handle) {
        let uuid = task.uuid;
        let sender = self.sender.clone();

        let heartbeat = self.gen_heartbeat_future(task, handle);
        let f = heartbeat.then(move |r| {
            let msg = Message::HeartbeatResponse(uuid, r);
            //worker was droped before loop routine, so it is safe to unwrap.
            sender.send(msg).unwrap();
            Ok(())
        });
        handle.spawn(f);
    }
}

enum Message<Q> {
    HeartbeatRequest(HeartbeatTask),
    HeartbeatResponse(Uuid, Result<Q, Error>),
    Stop,
}

type Targets<P, Q> = Arc<Mutex<HashMap<Uuid, Target<P, Q>>>>;

pub struct Hub<P, Q> {
    targets: Targets<P, Q>,
    sender: Sender<Message<Q>>,
    worker: Option<FutureWorker<HeartbeatTask>>,
    thread_handle: Option<JoinHandle<()>>,
}

struct Inner<P, Q> {
    targets: Targets<P, Q>,
    sender: Sender<Message<Q>>,
    receiver: Receiver<Message<Q>>,
    scheduler: Scheduler<HeartbeatTask>,
}

impl<P, Q> Hub<P, Q>
where
    P: ProtoMessage,
    Q: MessageStatic,
{
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        let worker = FutureWorker::new(
            "heartbeat_hub_worker",
            HeartbeatRunner { sender: tx.clone() },
        );
        let scheduler = worker.get_scheduler();

        let mut hub = Hub {
            targets: Arc::new(Mutex::new(HashMap::new())),
            sender: tx,
            worker: Some(worker),
            thread_handle: None,
        };

        let inner = Inner {
            targets: Arc::clone(&hub.targets),
            sender: hub.sender.clone(),
            receiver: rx,
            scheduler: scheduler,
        };

        let thread_handle = thread::Builder::new()
            .name("heartbeat_hub".to_string())
            .spawn(move || Self::begin_loop(inner))
            .unwrap(); //TODO
        hub.thread_handle = Some(thread_handle);
        hub
    }

    pub fn add_target(&self, target: Target<P, Q>) -> Uuid {
        let uuid = target.uuid;
        let task = target.gen_task();
        let msg = Message::HeartbeatRequest(task);
        self.sender.send(msg).unwrap();

        let mut targets = self.targets.lock().unwrap();
        targets.insert(target.uuid, target);
        uuid
    }

    pub fn remove_target(&self, id: Uuid) {
        let mut targets = self.targets.lock().unwrap();
        targets.remove(&id);
    }

    fn begin_loop(inner: Inner<P, Q>) {
        loop {
            match inner.receiver.recv().unwrap() {
                Message::HeartbeatRequest(task) => {
                    if inner.scheduler.schedule(task).is_err() {
                        info!("detect worker scheduler stoped");
                        break;
                    }
                }
                Message::HeartbeatResponse(uuid, res) => {
                    let mut targets = inner.targets.lock().unwrap();
                    if let Some(target) = targets.remove(&uuid) {
                        let is_ok = res.is_ok();
                        (target.cb)(uuid, res);
                        if is_ok {
                            let task = target.gen_task();
                            let msg = Message::HeartbeatRequest(task);
                            inner.sender.send(msg).unwrap();
                            targets.insert(uuid, target);
                        } else {
                            warn!("heartbeat to {:?} failed!, remove target", target);
                        }
                    }
                }
                Message::Stop => break,
            }
        }
    }
}

impl<P, Q> Default for Hub<P, Q>
where
    P: ProtoMessage,
    Q: MessageStatic,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<P, Q> Drop for Hub<P, Q> {
    fn drop(&mut self) {
        //drop worker first.
        self.worker.take().unwrap();

        //exit loop routine;
        self.sender.send(Message::Stop).unwrap();
        let thread_handle = self.thread_handle.take().unwrap();
        thread_handle.join().unwrap();
    }
}
