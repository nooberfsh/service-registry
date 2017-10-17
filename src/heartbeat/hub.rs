use std::io;
use std::fmt;
use std::net::SocketAddr;
use std::time::Duration;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::thread::{self, JoinHandle};
use std::sync::mpsc::{self, Sender, Receiver};
use std::sync::atomic::{AtomicBool, Ordering};
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

type Cb<Q> = Box<Fn(Uuid, Result<Q, Error>) + Send + 'static>;

pub struct TargetBuilder<P, Q> {
    addr: SocketAddr,
    uuid: Uuid,
    interval: Option<Duration>,
    timeout: Option<Duration>,
    request: Option<P>,
    cb: Option<Cb<Q>>,
}

impl<P, Q> TargetBuilder<P, Q>
where
    P: ProtoMessage,
    Q: MessageStatic,
{
    pub fn new(addr: &SocketAddr) -> Self {
        TargetBuilder {
            addr: *addr,
            uuid: Uuid::new_v4(),
            interval: None,
            timeout: None,
            request: None,
            cb: None,
        }
    }

    pub fn request(mut self, request: P) -> Self {
        self.request = Some(request);
        self
    }

    pub fn interval(mut self, interval: Duration) -> Self {
        self.interval = Some(interval);
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn cb<F>(mut self, cb: F) -> Self
    where
        F: Fn(Uuid, Result<Q, Error>) + Send + 'static,
    {
        self.cb = Some(Box::new(cb));
        self
    }

    pub fn build(self) -> Result<Target<P, Q>, Error> {
        let payload = {
            if let Some(request) = self.request {
                Some(request
                    .write_to_bytes()
                    .map_err(|e| {
                        let s = format!("{:?}", e);
                        Error::SerializeFailed(s)
                    })
                    .and_then(|v| if v.is_empty() {
                        Err(Error::ZeroPayload)
                    } else {
                        Ok(v)
                    })?)
            } else {
                None
            }
        };
        Ok(Target {
            addr: self.addr,
            uuid: self.uuid,
            interval: self.interval,
            timeout: self.timeout,
            payload: payload,
            cb: self.cb,
            _marker: PhantomData,
        })
    }
}

pub struct Target<P, Q> {
    addr: SocketAddr,
    uuid: Uuid,
    interval: Option<Duration>,
    timeout: Option<Duration>,
    payload: Option<Vec<u8>>,
    cb: Option<Cb<Q>>,
    _marker: PhantomData<P>,
}

impl<P, Q> Target<P, Q>
where
    P: ProtoMessage,
    Q: MessageStatic,
{
    pub fn new(addr: &SocketAddr) -> Self {
        TargetBuilder::new(addr).build().unwrap()
    }

    pub fn get_id(&self) -> Uuid {
        self.uuid
    }

    pub fn get_interval(&self) -> Option<Duration> {
        self.interval
    }

    pub fn get_timeout(&self) -> Option<Duration> {
        self.timeout
    }

    pub fn get_payload(&self) -> &Option<Vec<u8>> {
        &self.payload
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
    timeout: Option<Duration>,
    payload: Option<Vec<u8>>,
}

struct HeartbeatRunner<Q> {
    sender: Sender<Message<Q>>,
    timeout: Duration,
    payload: Vec<u8>,
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
        let payload = task.payload.unwrap_or_else(|| self.payload.clone()).into();
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

        Timeout::new(task.timeout.unwrap_or_else(|| self.timeout), handle)
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
                            Either::B((e, _)) => Err(Error::IoErr(Arc::new(e))),
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
    handle: HubHandle<P, Q>,
    worker: Option<FutureWorker<HeartbeatTask>>,
    timer: Option<Timer>,
    thread_handle: Option<JoinHandle<()>>,
}

struct Inner<P, Q> {
    handle: HubHandle<P, Q>,
    receiver: Receiver<Message<Q>>,
    scheduler: FutureScheduler<HeartbeatTask>,
    timer_handle: TimerHandle,
    interval: Duration,
    cb: Option<Cb<Q>>,
}

pub struct HubBuilder<P, Q> {
    cb: Option<Cb<Q>>,
    timeout: Duration,
    interval: Duration,
    request: P,
}

impl<P, Q> HubBuilder<P, Q>
where
    P: ProtoMessage,
    Q: MessageStatic,
{
    pub fn new(request: P) -> Self {
        HubBuilder {
            cb: None,
            timeout: Duration::from_secs(5),
            interval: Duration::from_secs(1),
            request: request,
        }
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn interval(mut self, interval: Duration) -> Self {
        self.interval = interval;
        self
    }

    pub fn cb<F>(mut self, cb: F) -> Self
    where
        F: Fn(Uuid, Result<Q, Error>) + Send + 'static,
    {
        self.cb = Some(Box::new(cb));
        self
    }

    pub fn build(self) -> Result<Hub<P, Q>, Error> {
        let payload = self.request
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

        let (tx, rx) = mpsc::channel();
        let worker = FutureWorker::new(
            "heartbeat_hub_worker",
            HeartbeatRunner {
                sender: tx.clone(),
                payload: payload,
                timeout: self.timeout,
            },
        );
        let scheduler = worker.get_scheduler();

        let timer = Timer::new("hub_timer");
        let timer_handle = timer.get_handle();

        let hub_handle = HubHandle {
            valid: Arc::new(AtomicBool::new(true)),
            targets: Arc::new(Mutex::new(HashMap::new())),
            sender: tx,
        };

        let mut hub = Hub {
            handle: hub_handle,
            worker: Some(worker),
            timer: Some(timer),
            thread_handle: None,
        };

        let inner = Inner {
            handle: hub.handle.clone(),
            receiver: rx,
            scheduler: scheduler,
            timer_handle: timer_handle,
            interval: self.interval,
            cb: self.cb,
        };

        let thread_handle = thread::Builder::new()
            .name("heartbeat_hub".to_string())
            .spawn(move || Hub::begin_loop(inner))
            .unwrap(); //TODO
        hub.thread_handle = Some(thread_handle);
        Ok(hub)
    }
}

impl<P, Q> Hub<P, Q>
where
    P: ProtoMessage,
    Q: MessageStatic,
{
    pub fn new(request: P) -> Result<Self, Error> {
        HubBuilder::new(request).build()
    }

    pub fn get_handle(&self) -> HubHandle<P, Q> {
        self.handle.clone()
    }

    pub fn add_target(&self, target: Target<P, Q>) -> Uuid {
        self.handle.add_target(target).unwrap()
    }

    pub fn remove_target(&self, id: Uuid) -> Option<Target<P, Q>> {
        self.handle.remove_target(id).unwrap()
    }

    fn begin_loop(inner: Inner<P, Q>) {
        loop {
            match inner.receiver.recv().unwrap() {
                Message::HeartbeatRequest(task) => {
                    if inner.scheduler.schedule(task).is_err() {
                        info!("detect worker scheduler stoped");
                    }
                }
                Message::HeartbeatResponse(uuid, res) => {
                    let mut targets = inner.handle.targets.lock().unwrap();
                    if let Some(target) = targets.remove(&uuid) {
                        let is_ok = res.is_ok();
                        target.cb.as_ref().map(|cb| cb(uuid, res.clone()));
                        inner.cb.as_ref().map(|cb| cb(uuid, res));
                        if is_ok {
                            let sender = inner.handle.sender.clone();
                            let f = move || {
                                let msg = Message::WakeupTarget(uuid);
                                sender.send(msg).unwrap();
                            };
                            if inner
                                .timer_handle
                                .timeout(target.interval.unwrap_or_else(|| inner.interval), f)
                                .is_err()
                            {
                                info!("detect worker scheduler stoped");
                            }
                            targets.insert(uuid, target);
                        } else {
                            warn!("heartbeat to {:?} failed!, remove target", target);
                        }
                    }
                }
                Message::WakeupTarget(uuid) => {
                    let targets = inner.handle.targets.lock().unwrap();
                    if let Some(target) = targets.get(&uuid) {
                        let task = target.gen_task();
                        let msg = Message::HeartbeatRequest(task);
                        inner.handle.sender.send(msg).unwrap();
                    }
                }
                Message::Stop => break,
            }
        }
    }
}

pub struct HubHandle<P, Q> {
    valid: Arc<AtomicBool>,
    targets: Targets<P, Q>,
    sender: Sender<Message<Q>>,
}

impl<P, Q> HubHandle<P, Q>
where
    P: ProtoMessage,
    Q: MessageStatic,
{
    pub fn add_target(&self, target: Target<P, Q>) -> Result<Uuid, Error> {
        if !self.valid.load(Ordering::SeqCst) {
            return Err(Error::HubStopped);
        }
        let uuid = target.uuid;
        let task = target.gen_task();
        let msg = Message::HeartbeatRequest(task);
        self.sender.send(msg).unwrap();

        let mut targets = self.targets.lock().unwrap();
        targets.insert(target.uuid, target);
        Ok(uuid)
    }

    pub fn remove_target(&self, id: Uuid) -> Result<Option<Target<P, Q>>, Error> {
        if !self.valid.load(Ordering::SeqCst) {
            return Err(Error::HubStopped);
        }
        let mut targets = self.targets.lock().unwrap();
        Ok(targets.remove(&id))
    }
}

impl<P, Q> Clone for HubHandle<P, Q> {
    fn clone(&self) -> Self {
        HubHandle {
            valid: Arc::clone(&self.valid),
            targets: Arc::clone(&self.targets),
            sender: self.sender.clone(),
        }
    }
}

impl<P, Q> Drop for Hub<P, Q> {
    fn drop(&mut self) {
        //drop worker first.
        self.worker.take().unwrap();

        //drop timer
        self.timer.take().unwrap();

        self.handle.valid.store(false, Ordering::SeqCst);
        //exit loop routine;
        self.handle.sender.send(Message::Stop).unwrap();
        let thread_handle = self.thread_handle.take().unwrap();
        thread_handle.join().unwrap();
    }
}
