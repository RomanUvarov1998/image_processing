use std::{sync::{Arc, Condvar, Mutex, MutexGuard}, thread::{self, JoinHandle}};
use super::{TaskMsg, guarded::Guarded, progress_provider::HaltMessage, task::TaskBase};


pub struct BackgroundWorker {
    inner: Arc<Inner>,
    tx_halt: std::sync::mpsc::SyncSender<HaltMessage>,
    _processing_thread_handle: JoinHandle<()>
}

impl BackgroundWorker {
    pub fn new(progress_tx: std::sync::mpsc::Sender<TaskMsg>) -> Self {
        let (tx_halt, rx_halt) = std::sync::mpsc::sync_channel::<HaltMessage>(1);
        
        let inner = Arc::new(Inner::new(progress_tx, rx_halt));

        let inner_arc = Arc::clone(&inner);
        let _processing_thread_handle: JoinHandle<()> = thread::Builder::new()
            .name("Processing".to_string())
            .spawn(move || 
        {
            loop {
                let mut guard = inner_arc.guarded.lock().expect("Couldn't lock");

                let condition = |g: &mut Guarded| !g.has_task_to_do();
                guard = inner_arc.cv.wait_while(guard, condition).expect("Couldn't wait");

                guard.do_task_and_save_result();
            }
        })
            .expect("Couldn't create a processing thread");

        BackgroundWorker { inner, tx_halt, _processing_thread_handle }
    }

    pub fn locked(&self) -> MutexGuard<Guarded> {
        self.inner.guarded.lock().expect("Couldn't lock")
    }

    pub fn start_task(&mut self, task: TaskBase) {
        self.locked().start_task(task);
        self.inner.cv.notify_one();
    }
    
    pub fn halt_processing(&mut self) {
        use std::sync::mpsc::TrySendError;

        if let Err(err) = self.tx_halt.try_send(HaltMessage) {
            if let TrySendError::Disconnected(_) = err {
                panic!("Rx_halt disconnected");
            }
        }
    }
}


struct Inner {
    cv: Condvar,
    guarded: Mutex<Guarded>
}

impl Inner {
    fn new(progress_tx: std::sync::mpsc::Sender<TaskMsg>, rx_halt: std::sync::mpsc::Receiver<HaltMessage>) -> Self {
        Inner {
            cv: Condvar::new(),
            guarded: Mutex::new(Guarded::new(progress_tx, rx_halt))
        }
    }
}
