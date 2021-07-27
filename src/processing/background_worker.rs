use super::{
    guarded::{Guarded, TaskSetup},
    task_info_channel::ExecutorHandle,
};
use std::{
    sync::{Arc, Condvar, Mutex, MutexGuard},
    thread::{self, JoinHandle},
};

pub struct BackgroundWorker {
    inner: Arc<Inner>,
    _processing_thread_handle: JoinHandle<()>,
}

impl BackgroundWorker {
    pub fn new(executor_handle: ExecutorHandle) -> Self {
        let inner = Arc::new(Inner::new(executor_handle));

        let inner_arc = Arc::clone(&inner);
        let _processing_thread_handle: JoinHandle<()> = thread::Builder::new()
            .name("Processing".to_string())
            .spawn(move || loop {
                let mut guard = inner_arc.guarded.lock().expect("Couldn't lock");

                let condition = |g: &mut Guarded| !g.has_task_to_do();
                guard = inner_arc
                    .cv
                    .wait_while(guard, condition)
                    .expect("Couldn't wait");

                guard.do_task_and_save_result();
            })
            .expect("Couldn't create a processing thread");

        BackgroundWorker {
            inner,
            _processing_thread_handle,
        }
    }

    pub fn locked(&self) -> MutexGuard<Guarded> {
        self.inner.guarded.lock().expect("Couldn't lock")
    }

    pub fn start_task(&mut self, setup: TaskSetup) {
        print!("notified ");

        self.locked().start_task(setup);
        self.inner.cv.notify_one();
    }
}

struct Inner {
    cv: Condvar,
    guarded: Mutex<Guarded>,
}

impl Inner {
    fn new(executor_handle: ExecutorHandle) -> Self {
        Inner {
            cv: Condvar::new(),
            guarded: Mutex::new(Guarded::new(executor_handle)),
        }
    }
}
