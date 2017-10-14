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
use worker::{FutureRunner, FutureWorker, FutureScheduler};

use super::Error;
use super::timer::{Timer, TimerHandle};

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

    pub fn get_id(&self) -> Uuid {
        self.uuid
    }

    fn gen_task(&self) -> HeartbeatTask {
        HeartbeatTask {
            addr: self.addr,
            uuid: self.uuid,
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
        let base = TcpStream::connect(&task.addr, handle)
            .and_then(move |stream| {
                let frame = Framed::new(stream);
                frame.send(payload)
            })
            .and_then(move |stream: Framed<_>| {
                stream.into_future().map_err(|(e, _)| e)
            })
            .and_then(|(item, _)| {
                item.ok_or_else(|| io::Error::new(io::ErrorKind::Other, "closed by server"))
            })
            .and_then(|r| parse_from_bytes::<Q>(&r).map_err(From::from));

        Timeout::new(task.timeout, handle)
            .unwrap()
            .select2(base)
            .then(|r| {
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

impl<Q> FutureRunner<HeartbeatTask> for HeartbeatRunner<Q>
where
    Q: MessageStatic + 'static,
{
    fn run(&mut self, task: HeartbeatTask, handle: &Handle) {
        let uuid = task.uuid;
        let sender = self.sender.clone();
        let f = self.gen_heartbeat_future(task, handle).then(move |r| {
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
    WakeupTarget(Uuid),
    Stop,
}

type Targets<P, Q> = Arc<Mutex<HashMap<Uuid, Target<P, Q>>>>;

pub struct Hub<P, Q> {
    targets: Targets<P, Q>,
    sender: Sender<Message<Q>>,
    worker: Option<FutureWorker<HeartbeatTask>>,
    timer: Option<Timer>,
    thread_handle: Option<JoinHandle<()>>,
}

struct Inner<P, Q> {
    targets: Targets<P, Q>,
    sender: Sender<Message<Q>>,
    receiver: Receiver<Message<Q>>,
    scheduler: FutureScheduler<HeartbeatTask>,
    timer_handle: TimerHandle,
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

        let timer = Timer::new("hub_timer");
        let timer_handle = timer.get_handle();

        let mut hub = Hub {
            targets: Arc::new(Mutex::new(HashMap::new())),
            sender: tx,
            worker: Some(worker),
            timer: Some(timer),
            thread_handle: None,
        };

        let inner = Inner {
            targets: Arc::clone(&hub.targets),
            sender: hub.sender.clone(),
            receiver: rx,
            scheduler: scheduler,
            timer_handle: timer_handle,
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

    pub fn remove_target(&self, id: Uuid) -> Option<Target<P, Q>> {
        let mut targets = self.targets.lock().unwrap();
        targets.remove(&id)
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
                            let sender = inner.sender.clone();
                            let f = move || {
                                let msg = Message::WakeupTarget(uuid);
                                sender.send(msg).unwrap();
                            };
                            if inner.timer_handle.timeout(target.interval, f).is_err() {
                                info!("detect worker scheduler stoped");
                                break;
                            }
                            targets.insert(uuid, target);
                        } else {
                            warn!("heartbeat to {:?} failed!, remove target", target);
                        }
                    }
                }
                Message::WakeupTarget(uuid) => {
                    let targets = inner.targets.lock().unwrap();
                    if let Some(target) = targets.get(&uuid) {
                        let task = target.gen_task();
                        let msg = Message::HeartbeatRequest(task);
                        inner.sender.send(msg).unwrap();
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

        //drop timer
        self.timer.take().unwrap();

        //exit loop routine;
        self.sender.send(Message::Stop).unwrap();
        let thread_handle = self.thread_handle.take().unwrap();
        thread_handle.join().unwrap();
    }
}
