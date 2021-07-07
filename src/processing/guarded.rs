use fltk::{image::RgbImage};
use crate::{img::{Img, PixelsArea}, my_err::MyError};
use super::{FilterBase, TaskMsg, progress_provider::HaltMessage, task_info::*};

pub struct Guarded {
	pub tx_notify: std::sync::mpsc::Sender<TaskMsg>,
	pub rx_halt: std::sync::mpsc::Receiver<HaltMessage>,
    pub task: Option<TaskBase>,
    pub initial_step: Step,
    pub proc_steps: Vec<ProcStep>,
}

impl Guarded {
	pub fn new(progress_tx: std::sync::mpsc::Sender<TaskMsg>, rx_halt: std::sync::mpsc::Receiver<HaltMessage>) -> Self {
		Guarded {
			tx_notify: progress_tx, rx_halt,
			task: None,
			initial_step: Step { img: None },
			proc_steps: Vec::new()
		}
	}

	pub fn has_task_to_do(&self) -> bool {
		if let Some(ref task) = self.task {
			!task.is_completed()
		} else {
			false
		}
	}

	pub fn do_task_and_save_result(&mut self) {
		// make the halt message buffer clean
		while let Ok(_) = self.rx_halt.try_recv() {}

        let mut task: TaskBase = self.task.take().unwrap();

        task.complete(self);

        self.task = Some(task);
        
        self.tx_notify.send( TaskMsg::Finished ).unwrap();
	}



	pub fn start_import(&mut self, path: String) {
        self.put_task(ImportTask::new(path));
	}

    pub fn has_initial_img(&self) -> bool {
        self.initial_step.img.is_some()
    }

    pub fn get_init_img_drawable(&self) -> RgbImage {
        self.initial_step.img.as_ref().unwrap().get_drawable_copy()
    }
    
    pub fn get_init_img_descr(&self) -> String {
        self.initial_step.img.as_ref().unwrap().get_description()
    }

	
    pub fn add_step(&mut self, filter: FilterBase) {
        self.proc_steps.push( 
            ProcStep { 
                img: None, 
                filter  
            } 
        );
    }

    pub fn edit_step(
        &mut self, 
        step_num: usize, 
        mut action: impl FnMut(&mut FilterBase) -> bool
    ) -> bool {
        action(&mut self.proc_steps[step_num].filter)
    }

    pub fn remove_step(&mut self, step_num: usize) {
        self.proc_steps.remove(step_num);
    }

	pub fn swap_steps(&mut self, step_num1: usize, step_num2: usize) {
		self.proc_steps.swap(step_num1, step_num2);
    }

    pub fn get_steps_count(&self) -> usize {
        self.proc_steps.len()
    }


    pub fn check_if_can_start_processing(&self, step_num: usize) -> StartProcResult {
        if self.initial_step.img.is_none() {
            StartProcResult::NoInitialImg
        } else if step_num > 0 && self.proc_steps[step_num - 1].img.is_none() {
            StartProcResult::NoPrevStepImg
        } else {
            StartProcResult::CanStart
        }
    }

    pub fn start_processing(
		&mut self, 
		step_num: usize, 
		crop_area: Option<PixelsArea>
	) {
        self.put_task(ProcTask::new(step_num, crop_area));
    }

    pub fn get_step_descr(&self, step_num: usize) -> String {
        self.proc_steps[step_num].get_description()
    }

	pub fn get_step_img_drawable(&self, step_num: usize) -> Option<RgbImage> {
		match self.proc_steps[step_num].img {
            Some(ref img) => Some(img.get_drawable_copy()),
            None => None,
        }
	}


    pub fn get_filter_params_as_str(&self, step_num: usize) -> Option<String> {
        self.proc_steps[step_num].filter.params_to_string()
    }
    
    pub fn get_filter_save_name(&self, step_num: usize) -> String {
		self.proc_steps[step_num].filter.get_save_name()
    }


    pub fn check_if_can_export(&self) -> StartResultsSavingResult {
        if self.proc_steps.len() == 0 {
            StartResultsSavingResult::NoSteps
        } else if self.proc_steps.iter().any(|s| s.img.is_none()) {
            StartResultsSavingResult::NotAllStepsHaveResult
        } else {
            StartResultsSavingResult::CanStart
        }
    }

    pub fn start_export(&mut self, dir_path: String) {
		self.put_task(ExportTask::new(dir_path))
    }


    pub fn get_task_result(&mut self) -> Result<(), MyError> {
        self.task.take().unwrap().take_result()
    }


	pub fn try_get_initial_img(&self) -> &Img {
        self.initial_step.img.as_ref().unwrap()
	}
	
	pub fn try_get_step_img(&self, step_num: usize) -> &Img {
        self.proc_steps[step_num].img.as_ref().unwrap()
	}

	fn put_task(&mut self, task: TaskBase) {
        assert!(self.task.is_none());
        self.task = Some(task);
	}
}


pub struct Step { pub img: Option<Img> }


pub struct ProcStep {
    pub img: Option<Img>,
    pub filter: FilterBase
}

impl ProcStep {
    pub fn get_description(&self) -> String {
        let filter_descr = self.filter.get_description();

        let img_descr = match self.img {
            Some(ref img) => img.get_description(),
            None => String::new(),
        };

        format!("{} {}", &filter_descr, &img_descr)
    }
}



pub enum StartProcResult { NoInitialImg, NoPrevStepImg, CanStart }


pub enum StartResultsSavingResult { NoSteps, NotAllStepsHaveResult, CanStart }