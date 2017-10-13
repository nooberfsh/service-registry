use std::time::Duration;
use std::boxed::FnBox;

use futures::Future;
use tokio_core::reactor::{Handle, Timeout};

use future_worker::{Runner, FutureWorker, Scheduler, Stopped};

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
    fn run(&mut self, task: Task, handle: &Handle) {
        let f = Timeout::new(task.timeout, handle).unwrap().then(move |_| {
            Ok((task.cb)())
        });
        handle.spawn(f);
    }
}

pub struct Timer {
    worker: FutureWorker<Task>,
}

pub struct TimerHandle {
    scheduler: Scheduler<Task>,
}

impl Timer {
    pub fn new<N: Into<String>>(name: N) -> Self {
        Timer { worker: FutureWorker::new(name, TimerImpl) }
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
