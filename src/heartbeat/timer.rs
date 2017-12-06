use std::time::Duration;
use std::boxed::FnBox;

use futures::Future;
use tokio_core::reactor::{Handle, Timeout};

use worker::Stopped;
use worker::future::{Runner, Worker, Scheduler, BoxFuture};

struct Task {
    timeout: Duration,
    cb: Box<FnBox() + Send + 'static>,
}

impl Task {
    fn new<F>(timeout: Duration, cb: F) -> Self
    where
        F: FnOnce() + Send + 'static,
    {
        Task {
            timeout: timeout,
            cb: Box::new(cb),
        }
    }
}

struct TimerImpl;

impl Runner<Task> for TimerImpl {
    fn future(&self, task: Task, handle: &Handle) -> BoxFuture {
        let f = Timeout::new(task.timeout, handle).unwrap().then(move |_| {
            Ok((task.cb)())
        });
        Box::new(f)
    }
}

pub struct Timer {
    worker: Worker<Task>,
}

pub struct TimerHandle {
    scheduler: Scheduler<Task>,
}

impl Timer {
    pub fn new<N: Into<String>>(name: N) -> Self {
        Timer { worker: Worker::new(name, TimerImpl) }
    }

    #[allow(dead_code)]
    pub fn timeout<F>(&self, timeout: Duration, cb: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let task = Task::new(timeout, cb);
        self.worker.schedule(task);
    }

    pub fn get_handle(&self) -> TimerHandle {
        TimerHandle { scheduler: self.worker.get_scheduler() }
    }
}

impl TimerHandle {
    pub fn timeout<F>(&self, timeout: Duration, cb: F) -> Result<(), Stopped>
    where
        F: FnOnce() + Send + 'static,
    {
        let task = Task::new(timeout, cb);
        self.scheduler.schedule(task)
    }
}
