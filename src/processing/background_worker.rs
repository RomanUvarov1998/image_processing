use std::{sync::{Arc, Condvar, Mutex, MutexGuard}, thread::{self, JoinHandle}};
use fltk::{image::RgbImage, prelude::ImageExt};

use crate::{img::{Img, PixelsArea}, message::{Msg, Proc}, my_err::MyError};
use super::{FilterBase, progress_provider::{HaltMessage, ProgressProvider}};


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

                guard = inner_arc.cv.wait_while(guard, |g| g.has_task() == false).expect("Couldn't wait");

                let task = guard.take_task();
                let step_num = task.step_num;

                let mut prog_prov = ProgressProvider::new(
                    &progress_tx, 
                    &rx_halt, 
                    step_num,
                    guard.proc_steps.len()
                );

                // leave the message buffer clean
                while let Ok(_) = rx_halt.try_recv() {}
                
                let step = &guard.proc_steps[step_num];
                let mut img_to_process = if step_num == 0 {
                    guard.initial_step.img.as_ref().unwrap()
                } else {
                    guard.proc_steps[step_num - 1].step.img.as_ref().unwrap()
                };

                let cropped_copy: Option<Img>;
                if let Some(crop_area) = task.crop_area {
                    cropped_copy = Some(img_to_process.get_cropped_copy(crop_area));
                    img_to_process = cropped_copy.as_ref().unwrap();
                }

                let img_result = match step.filter.filter(&img_to_process, &mut prog_prov) {
                    Ok(img) => {
                        assert!(prog_prov.all_actions_completed());
                        Some(img)
                    },
                    Err(_halted) => None
                };

                let task_result = ProcResult {
                    img: match img_result {
                        Some(ref img) => Some(img.get_drawable_copy()),
                        None => None,
                    },
                    it_is_the_last_step: task.step_num == guard.proc_steps.len() - 1,
                    processing_was_halted: img_result.is_none(),
                };

                guard.proc_steps[task.step_num].step.img = img_result;

                guard.put_result(task_result);

                drop(guard);

                progress_tx.send(Msg::Proc(Proc::CompletedStep { num: task.step_num }));
            }
        })
            .expect("Couldn't create a processing thread");

        BackgroundWorker { inner, tx_halt, _processing_thread_handle }
    }


    pub fn load_initial_img(&mut self, path: &str) -> Result<(), MyError> {
        let mut guard = self.get_unlocked_guard();

        let sh_im = fltk::image::SharedImage::load(path)?;

        if sh_im.w() < 0 { return Err(MyError::new("Ширина загруженного изображения < 0".to_string())); }
        if sh_im.h() < 0 { return Err(MyError::new("Высота загруженного изображения < 0".to_string())); }

        let img = Img::from(sh_im);

        guard.initial_step.img = Some(img);

        Ok(())
    }

    pub fn has_initial_img(&self) -> bool {
        let guard = self.get_unlocked_guard();
        guard.initial_step.img.is_some()
    }

    pub fn get_init_img_drawable(&self) -> RgbImage {
        let guard = self.get_unlocked_guard();
        guard.initial_step.img.as_ref().unwrap().get_drawable_copy()
    }
    
    pub fn get_init_img_descr(&self) -> String {
        let guard = self.get_unlocked_guard();
        guard.initial_step.img.as_ref().unwrap().get_description()
    }


    pub fn add_step(&mut self, filter: FilterBase) {
        let mut guard = self.get_unlocked_guard();
        guard.proc_steps.push( 
            ProcStep { 
                step: Step { img: None }, 
                filter  
            } 
        );
    }

    pub fn edit_step(
        &mut self, 
        step_num: usize, 
        mut action: impl FnMut(&mut FilterBase) -> bool
    ) -> bool {
        let mut guard = self.get_unlocked_guard();
        action(&mut guard.proc_steps[step_num].filter)
    }

    pub fn remove_step(&mut self, step_num: usize) {
        let mut guard = self.get_unlocked_guard();
        guard.proc_steps.remove(step_num);
    }


    pub fn check_if_can_start_processing(&self, step_num: usize) -> StartProcResult {
        let guard = self.get_unlocked_guard();

        if guard.initial_step.img.is_none() {
            StartProcResult::NoInitialImg
        } else if step_num > 0 && guard.proc_steps[step_num - 1].step.img.is_none() {
            StartProcResult::NoPrevStepImg
        } else {
            StartProcResult::CanStart
        }
    }

    pub fn start_processing(&mut self, step_num: usize, crop_area: Option<PixelsArea>) {
        let mut guard = self.get_unlocked_guard();
        guard.put_task( ProcTask { step_num, crop_area } );
        drop(guard);

        self.inner.cv.notify_one();
    }

    pub fn halt_processing(&mut self) {
        use std::sync::mpsc::TrySendError;

        match self.tx_halt.try_send(HaltMessage) {
            Ok(()) => {},
            Err(err) => match err {
                TrySendError::Full(_) => {},
                TrySendError::Disconnected(_) => panic!("Halt message rx was destroyed, but tx is still trying to send"),
            },
        }
    }

    pub fn get_result(&mut self) -> ProcResult {
        let mut guard = self.get_unlocked_guard();
        guard.take_task_result()
    }

    pub fn get_step_descr(&self, step_num: usize) -> String {
        let guard = self.get_unlocked_guard();
        guard.proc_steps[step_num].get_description()
    }


    pub fn get_filter_params_as_str(&self, step_num: usize) -> Option<String> {
        let guard = self.get_unlocked_guard();
        guard.proc_steps[step_num].filter.params_to_string()
    }
    
    pub fn get_filter_save_name(&self, step_num: usize) -> String {
        let guard = self.get_unlocked_guard();
        guard.proc_steps[step_num].filter.get_save_name()
    }


    pub fn all_steps_have_result(&self) -> bool {
        let guard = self.get_unlocked_guard();
        guard.proc_steps.iter().all(|s| s.step.img.is_some())
    }

    pub fn try_save_img_to_file(&self, step_num: usize, path: &str) -> Result<(), MyError> {
        let guard = self.get_unlocked_guard();
        guard.proc_steps[step_num].step.img.as_ref().unwrap().try_save(path)
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
            guarded: Mutex::new(
                Guarded {
                    exch: ThreadExchange::Empty,
                    initial_step: Step { img: None },
                    proc_steps: Vec::new()
                }
            )
        }
    }
}


struct Guarded {
    exch: ThreadExchange,
    initial_step: Step,
    proc_steps: Vec<ProcStep>,
}

impl Guarded {
    fn has_task(&mut self) -> bool {
        match self.exch {
            ThreadExchange::Empty | ThreadExchange::HasResult (_) => false,
            ThreadExchange::HasTask (_) => true,
        }
    }

    fn put_task(&mut self, task: ProcTask) {
        self.exch = ThreadExchange::HasTask(Some(task));
    }

    fn take_task(&mut self) -> ProcTask {
        match self.exch {
            ThreadExchange::Empty | ThreadExchange::HasResult (_) => unreachable!(),
            ThreadExchange::HasTask (ref mut task) => task.take().expect("didn't found task"),
        }
    }

    fn put_result(&mut self, task_result: ProcResult) {
        self.exch = ThreadExchange::HasResult(Some(task_result));
    }

    fn take_task_result(&mut self) -> ProcResult {
        match self.exch {
            ThreadExchange::Empty | ThreadExchange::HasTask (_) => unreachable!(),
            ThreadExchange::HasResult (ref mut result) => result.take().expect("didn't found task result"),
        }
    }
}


enum ThreadExchange {
    Empty,
    HasTask(Option<ProcTask>),
    HasResult(Option<ProcResult>)
}


struct ProcTask { step_num: usize, crop_area: Option<PixelsArea> }

#[derive(Debug)]
pub struct ProcResult { 
    img: Option<RgbImage>, 
    it_is_the_last_step: bool,
    processing_was_halted: bool
}

impl ProcResult {
    pub fn it_is_the_last_step(&self) -> bool {
        self.it_is_the_last_step
    }

    pub fn processing_was_halted(&self) -> bool {
        self.processing_was_halted
    }

    pub fn get_image(&mut self) -> Option<RgbImage> {
        self.img.take()
    }
}


struct Step { img: Option<Img> }


struct ProcStep {
    step: Step,
    filter: FilterBase
}

impl ProcStep {
    fn get_description(&self) -> String {
        let filter_descr = self.filter.get_description();

        let img_descr = match self.step.img {
            Some(ref img) => img.get_description(),
            None => String::new(),
        };

        format!("{} {}", &filter_descr, &img_descr)
    }
}


pub enum StartProcResult { NoInitialImg, NoPrevStepImg, CanStart }