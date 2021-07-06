use std::{sync::{Arc, Condvar, Mutex, MutexGuard}, thread::{self, JoinHandle}};
use fltk::{image::RgbImage};

use crate::{img::{PixelsArea}, message::Msg, my_err::MyError};
use super::{BWError, FilterBase, guarded::{Guarded, StartProcResult, StartResultsSavingResult}, progress_provider::{HaltMessage}, task_info::{ExportResult, ProcResult}};


pub struct BackgroundWorker {
    inner: Arc<Inner>,
    tx_halt: std::sync::mpsc::SyncSender<HaltMessage>,
    _processing_thread_handle: JoinHandle<()>
}

impl BackgroundWorker {
    pub fn new(progress_tx: fltk::app::Sender<Msg>) -> Self {
        let inner = Arc::new(Inner::new());

        let (tx_halt, rx_halt) = std::sync::mpsc::sync_channel::<HaltMessage>(1);

        let inner_arc = Arc::clone(&inner);
        let _processing_thread_handle: JoinHandle<()> = thread::Builder::new()
            .name("Processing".to_string())
            .spawn(move || 
        {
            loop {
                let mut guard = inner_arc.guarded.lock().expect("Couldn't lock");

                let condition = |g: &mut Guarded| !g.has_task_to_do();
                guard = inner_arc.cv.wait_while(guard, condition).expect("Couldn't wait");

                guard.do_task_and_save_result(&progress_tx, &rx_halt);
            }
        })
            .expect("Couldn't create a processing thread");

        BackgroundWorker { inner, tx_halt, _processing_thread_handle }
    }


    pub fn try_load_initial_img(&mut self, path: &str) -> Result<(), MyError> {
        let mut guard = self.get_unlocked_guard();
        guard.try_load_initial_img(path)
    }

    pub fn has_initial_img(&self) -> bool {
        let guard = self.get_unlocked_guard();
        guard.has_initial_img()
    }

    pub fn get_init_img_drawable(&self) -> Result<RgbImage, BWError> {
        let guard = self.get_unlocked_guard();
        guard.get_init_img_drawable()
    }
    
    pub fn get_init_img_descr(&self) -> Result<String, BWError> {
        let guard = self.get_unlocked_guard();
        guard.get_init_img_descr()
    }


    pub fn add_step(&mut self, filter: FilterBase) {
        let mut guard = self.get_unlocked_guard();
        guard.add_step(filter);
    }

    pub fn edit_step(
        &mut self, 
        step_num: usize, 
        action: impl FnMut(&mut FilterBase) -> bool
    ) -> Result<bool, BWError> {
        let mut guard = self.get_unlocked_guard();
        guard.edit_step(step_num, action)
    }

    pub fn remove_step(&mut self, step_num: usize) -> Result<(), BWError> {
        let mut guard = self.get_unlocked_guard();
        guard.remove_step(step_num)
    }


    pub fn check_if_can_start_processing(&self, step_num: usize) -> StartProcResult {
        let guard = self.get_unlocked_guard();
        guard.check_if_can_start_processing(step_num)
    }

    pub fn start_processing(&mut self, step_num: usize, crop_area: Option<PixelsArea>) -> Result<(), BWError> {
        let mut guard = self.get_unlocked_guard();
        guard.start_processing(step_num, crop_area)?;
        drop(guard);

        self.inner.cv.notify_one();

        Ok(())
    }

    pub fn halt_processing(&mut self) -> Result<(), BWError> {
        use std::sync::mpsc::TrySendError;

        match self.tx_halt.try_send(HaltMessage) {
            Ok(()) => Ok(()),
            Err(err) => match err {
                TrySendError::Full(_) => Ok(()),
                TrySendError::Disconnected(_) => Err( BWError::Custom { msg: "Rx was destroyed".to_string() } ),
            },
        }
    }

    pub fn get_proc_result(&mut self) -> Result<ProcResult, BWError> {
        let mut guard = self.get_unlocked_guard();
        guard.get_proc_result()
    }

    pub fn get_step_descr(&self, step_num: usize) -> Result<String, BWError> {
        let guard = self.get_unlocked_guard();
        guard.get_step_descr(step_num)
    }


    pub fn get_filter_params_as_str(&self, step_num: usize) -> Result<Option<String>, BWError> {
        let guard = self.get_unlocked_guard();
        guard.get_filter_params_as_str(step_num)
    }
    
    pub fn get_filter_save_name(&self, step_num: usize) -> Result<String, BWError> {
        let guard = self.get_unlocked_guard();
        guard.get_filter_save_name(step_num)
    }


    pub fn check_if_can_export(&self) -> StartResultsSavingResult {
        let guard = self.get_unlocked_guard();
        guard.check_if_can_export()
    }

    pub fn start_export(&self, dir_path: String) -> Result<(), BWError> {
        let mut guard = self.get_unlocked_guard();
        guard.start_export(dir_path)?;
        drop(guard);

        self.inner.cv.notify_one();

        Ok(())
    }

    pub fn get_export_result(&self) -> Result<ExportResult, BWError> {
        let mut guard = self.get_unlocked_guard();
        guard.get_export_result()
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
    fn new() -> Self {
        Inner {
            cv: Condvar::new(),
            guarded: Mutex::new(Guarded::new())
        }
    }
}
