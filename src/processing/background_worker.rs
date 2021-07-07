use std::{sync::{Arc, Condvar, Mutex, MutexGuard}, thread::{self, JoinHandle}};
use crate::{img::{PixelsArea}, message::TaskMsg};
use super::{guarded::{Guarded, StartProcResult}, progress_provider::HaltMessage};


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

    pub fn unlocked(&self) -> MutexGuard<Guarded> {
        self.get_unlocked_guard()
    }


    pub fn check_if_can_start_processing(&self, step_num: usize) -> StartProcResult {
        let guard = self.get_unlocked_guard();
        guard.check_if_can_start_processing(step_num)
    }

    pub fn start_processing(&mut self, step_num: usize, crop_area: Option<PixelsArea>) {
        let mut guard = self.get_unlocked_guard();
        guard.start_processing(step_num, crop_area);
        drop(guard);

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

    pub fn start_export(&self, dir_path: String) {
        let mut guard = self.get_unlocked_guard();
        guard.start_export(dir_path);
        drop(guard);

        self.inner.cv.notify_one();
    }


    fn get_unlocked_guard(&self) -> MutexGuard<Guarded> {
        self.inner.guarded.lock().expect("Couldn't lock")
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
