use std::io;
use std::sync::Arc;
use std::sync::mpsc;
use std::net::{SocketAddr, SocketAddrV4};
use std::thread::{self, JoinHandle};

use bytes::BytesMut;
use tokio_core::reactor::{Core, Handle};
use tokio_core::net::TcpListener;
use tokio_io::codec::length_delimited::Framed;
use tokio_io::AsyncWrite;
use futures::sync::oneshot::{self, Sender, Receiver};
use futures::{Future, Stream, Sink};
use protobuf::{Message, MessageStatic};
use protobuf::core::parse_from_bytes;

type Handler<P, Q> = Arc<Fn(P) -> Q + Send + Sync + 'static>;

pub struct Server<P, Q> {
    name: String,
    shutdown_sender: Option<Sender<()>>,
    thread_handle: Option<JoinHandle<()>>,
    handler: Handler<P, Q>,
}

fn handle_bytes<P, Q, T>(
    bytes: BytesMut,
    framed: Framed<T>,
    handle: &Handle,
    handler: Handler<P, Q>,
) where
    P: Message + MessageStatic,
    Q: Message + MessageStatic,
    T: AsyncWrite + 'static,
{
    match parse_from_bytes::<P>(&bytes) {
        Ok(p) => {
            let v = handler(p).write_to_bytes().unwrap(); // TODO error handle;
            let f = framed.send(v.into()).map(|_| ()).map_err(|e| {
                warn!("send failed: {}", e)
            });
            handle.spawn(f);
        }
        Err(e) => warn!("{:?}", e),
    }
}

fn serve<P, Q>(
    shutdown_rx: Receiver<()>,
    handler: &Handler<P, Q>,
    listener: TcpListener,
    mut core: Core,
) where
    P: Message + MessageStatic,
    Q: Message + MessageStatic,
{
    let handle = core.handle();
    let server = listener
        .incoming()
        .for_each(|(stream, _)| {
            Framed::new(stream)
                .into_future()
                .map_err(|(e, _)| warn!("framed: {:?}", e))
                .map(|(bytes, framed)| match bytes {
                    Some(bytes) => handle_bytes(bytes, framed, &handle, handler.clone()),
                    None => warn!("closed by client"),
                })
                .then(|_| Ok(())) // always return Ok(()) to prevent server shutdown
        })
        .map_err(|e| error!("{:?}", e));

    // tx will drop after this worker thread.
    let server = shutdown_rx
        .map_err(|_| unreachable!())
        .select(server)
        .map_err(|(e, _)| {
            error!("heartbeat server encounter a fatal error, reason: {:?}", e)
        });

    core.run(server).unwrap();
}

impl<P, Q> Server<P, Q>
where
    P: 'static + Message + MessageStatic,
    Q: 'static + Message + MessageStatic,
{
    pub fn new<N, F>(n: N, f: F) -> Self
    where
        N: Into<String>,
        F: Fn(P) -> Q + Send + Sync + 'static,
    {
        Server {
            name: n.into(),
            shutdown_sender: None,
            thread_handle: None,
            handler: Arc::new(f),
        }
    }

    pub fn start(&mut self, port: u16) -> io::Result<()> {
        if self.shutdown_sender.is_some() {
            return Err(io::Error::new(io::ErrorKind::Other, "server was started"));
        }
        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        let (succeed_tx, succeed_rx) = mpsc::channel();
        let handler = self.handler.clone();

        let thread_handle = thread::Builder::new().name(self.name.clone()).spawn(
            move || {
                let core = Core::new().unwrap();
                let handle = core.handle();
                match Self::create_listener(handle, port) {
                    Ok(listener) => {
                        succeed_tx.send(Ok(())).unwrap();
                        info!("begin serve");
                        serve(shutdown_rx, &handler, listener, core);
                        info!("finish serve");
                    }
                    Err(e) => {
                        succeed_tx.send(Err(e)).unwrap();
                        warn!("create listener failed with port: {}", port);
                    }
                }
            },
        )?;

        succeed_rx.recv().unwrap().map(|_| {
            assert!(self.shutdown_sender.is_none());
            assert!(self.thread_handle.is_none());
            self.shutdown_sender = Some(shutdown_tx);
            self.thread_handle = Some(thread_handle);
        })
    }

    fn create_listener(handle: Handle, port: u16) -> io::Result<TcpListener> {
        let addr = SocketAddr::V4(SocketAddrV4::new("0.0.0.0".parse().unwrap(), port));
        TcpListener::bind(&addr, &handle)
    }
}

impl<P, Q> Drop for Server<P, Q> {
    fn drop(&mut self) {
        info!("begin to drop");
        self.shutdown_sender.take().map(|s| s.send(()).unwrap());
        self.thread_handle.take().map(|t| t.join().unwrap());
    }
}
