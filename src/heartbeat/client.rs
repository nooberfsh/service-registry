use std::io;
use std::net::SocketAddr;
use std::marker::PhantomData;
use std::sync::mpsc::{self, Sender, Receiver};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use futures::future::Either;
use futures::{Future, Sink, Stream};
use tokio_core::reactor::{Handle, Timeout};
use tokio_core::net::TcpStream;
use tokio_io::codec::length_delimited::Framed;
use protobuf::core::parse_from_bytes;
use protobuf::{Message as ProtoMessage, MessageStatic};
use uuid::Uuid;

use future_worker::{Runner, FutureWorker};
use super::HeartbeatItem;

type CallBack = Box<Fn(io::Result<Vec<u8>>) + Send + 'static>;

pub struct HeartbeatTask {
    addr: SocketAddr,
    payload: Vec<u8>,
    delay: Option<Duration>,
    cb: CallBack,
}

impl HeartbeatTask {
    pub fn new<T, F>(addr: SocketAddr, payload: T, delay: Option<Duration>, f: F) -> Self
    where
        T: Into<Vec<u8>>,
        F: Fn(io::Result<Vec<u8>>) + Send + 'static,
    {
        HeartbeatTask {
            addr: addr,
            payload: payload.into(),
            delay: delay,
            cb: Box::new(f),
        }
    }
}

pub struct HeartbeatRunner;

impl Runner<HeartbeatTask> for HeartbeatRunner {
    fn run(&mut self, task: HeartbeatTask, handle: &Handle) {
        let payload = task.payload;
        let cb = task.cb;
        let f = TcpStream::connect(&task.addr, handle).and_then(move |stream| {
            let frame = Framed::new(stream);
            frame.send(payload.into()).and_then(
                move |stream: Framed<_>| {
                    stream.into_future().map_err(|(e, _)| e).and_then(
                        |(item, _)| {
                            item.ok_or_else(
                                || io::Error::new(io::ErrorKind::Other, "closed by server"),
                            )
                        },
                    )
                },
            )
        });

        let df = match task.delay {
            Some(delay) => {
                let timeout = Timeout::new(delay, handle).unwrap();
                Either::A(timeout.then(|_| f))

            }
            None => Either::B(f),
        };

        let pf = df.then(move |r| {
            // TODO avoid copy
            cb(r.map(|t| t.to_vec()));
            Ok(())
        });
        handle.spawn(pf);
    }
}

enum Message {
    SendRequest(HeartbeatItem),
    RecvResponse(HeartbeatItem, io::Result<Vec<u8>>),
    Stop,
}

type Handler<Q> = Box<Fn(Uuid, io::Result<Q>) + Send + 'static>;
type Items = Arc<Mutex<HashMap<Uuid, HeartbeatItem>>>;

pub struct Client<P, Q> {
    name: String,
    interval: Duration,
    items: Items,
    sender: Sender<Message>,
    receiver: Option<Receiver<Message>>,
    thread_handle: Option<JoinHandle<()>>,
    handler: Option<Handler<Q>>,
    __request: PhantomData<P>,
}

struct Inner<Q> {
    interval: Duration,
    items: Items,
    sender: Sender<Message>,
    receiver: Receiver<Message>,
    handler: Handler<Q>,
    worker: FutureWorker<HeartbeatTask>,
}

impl<P: ProtoMessage, Q: MessageStatic + ProtoMessage> Client<P, Q> {
    pub fn new<N, F>(n: N, interval: Duration, f: F) -> Self
    where
        N: Into<String>,
        F: Fn(Uuid, io::Result<Q>) + Send + 'static,
    {
        let (tx, rx) = mpsc::channel();
        Client {
            name: n.into(),
            interval: interval,
            items: Default::default(),
            sender: tx,
            receiver: Some(rx),
            thread_handle: None,
            handler: Some(Box::new(f)),
            __request: PhantomData,
        }
    }

    fn inner(&mut self, worker: FutureWorker<HeartbeatTask>) -> Inner<Q> {
        Inner {
            interval: self.interval,
            items: self.items.clone(),
            sender: self.sender.clone(),
            receiver: self.receiver.take().unwrap(),
            handler: self.handler.take().unwrap(),
            worker: worker,
        }
    }

    pub fn start(&mut self, req: P) {
        assert_ne!(req.compute_size(), 0);

        let worker = FutureWorker::new("heartbeat_worker", HeartbeatRunner);
        let inner = self.inner(worker);

        let thread_handle = thread::Builder::new()
            .name(self.name.clone())
            .spawn(move || Self::begin_loop(inner, req))
            .unwrap();

        self.thread_handle = Some(thread_handle);
    }

    fn begin_loop(inner: Inner<Q>, req: P) {
        loop {
            let sender = inner.sender.clone();
            match inner.receiver.recv().unwrap() {
                Message::SendRequest(item) => {
                    // this cb will executed in worker thread, not this thread.
                    let cb = move |rsp| {
                        let msg = Message::RecvResponse(item, rsp);
                        // channel will be closed when dropping, so can not unwrap here
                        if sender.send(msg).is_err() {
                            return;
                        }
                    };
                    let v = req.write_to_bytes().unwrap(); // TODO error handle;
                    let t = HeartbeatTask::new(item.addr, v, Some(inner.interval), cb);
                    inner.worker.schedule(t);
                }
                Message::RecvResponse(item, rsp) => {
                    let r = rsp.and_then(|v| parse_from_bytes::<Q>(&v).map_err(From::from));
                    if r.is_ok() {
                        let lock = inner.items.lock().unwrap();
                        if !lock.contains_key(&item.uuid) {
                            info!("{:?} was removed", item.uuid);
                            continue;
                        }
                        drop(lock);
                        let msg = Message::SendRequest(item);
                        sender.send(msg).unwrap();
                    } else {
                        let mut lock = inner.items.lock().unwrap();
                        lock.remove(&item.uuid);
                    }
                    (inner.handler)(item.uuid, r);
                }
                Message::Stop => break,
            }
        }
    }

    pub fn add_service(&self, addr: SocketAddr) -> Uuid {
        let uuid = Uuid::new_v4();
        let item = HeartbeatItem::new(uuid, addr);
        {
            let mut lock = self.items.lock().unwrap();
            lock.insert(uuid, item);
        }
        let msg = Message::SendRequest(item);
        self.sender.send(msg).unwrap();
        uuid
    }

    pub fn remove_service(&self, uuid: Uuid) {
        let mut lock = self.items.lock().unwrap();
        lock.remove(&uuid);
    }
}

impl<P, Q> Drop for Client<P, Q> {
    fn drop(&mut self) {
        self.sender.send(Message::Stop).unwrap();
        self.thread_handle.take().map(|h| h.join().unwrap());
    }
}
