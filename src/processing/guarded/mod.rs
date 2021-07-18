use crate::{img::{Img, filter::FilterBase}, my_err::MyError};
use fltk::image::RgbImage;
use proc_step::ProcStep;
use super::ExecutorHandle;

mod proc_step;
pub mod tasks;


pub struct Guarded {
	executor_handle: ExecutorHandle,
    task: Option<tasks::TaskBase>,
    task_result: Option<Result<(), MyError>>,
    initial_img: Option<Img>,
    proc_steps: Vec<ProcStep>,
}

impl Guarded {
	pub fn new(executor_handle: ExecutorHandle) -> Self {
		Guarded {
			executor_handle,
			task: None, task_result: None,
			initial_img: None,
			proc_steps: Vec::new()
		}
	}

	pub fn has_task_to_do(&self) -> bool {
        self.task.is_some() && self.task_result.is_none()
	}

	pub fn do_task_and_save_result(&mut self) {
        let mut task: tasks::TaskBase = self.task.take().unwrap();
        self.task_result = Some(task.complete(self));
        self.executor_handle.assert_all_actions_completed();
	}
    
    pub fn start_task(&mut self, task: tasks::TaskBase) {
        assert!(self.task.is_none());
        self.task = Some(task);
	}

    pub fn get_task_result(&mut self) -> Result<(), MyError> {
        self.task_result.take().unwrap()
    }


    pub fn set_initial_img(&mut self, img: Img) {
        self.initial_img = Some(img);
        for step in self.proc_steps.iter_mut() {
            step.img = None;
        }
    }

	pub fn get_initial_img(&self) -> &Img {
        self.initial_img.as_ref().unwrap()
	}

    pub fn has_initial_img(&self) -> bool {
        self.initial_img.is_some()
    }

    pub fn get_init_img_drawable(&self) -> RgbImage {
        self.initial_img.as_ref().unwrap().get_drawable_copy()
    }
    
    pub fn get_init_img_descr(&self) -> String {
        self.initial_img.as_ref().unwrap().get_description()
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
	
	pub fn get_step_img(&self, step_num: usize) -> &Img {
        self.proc_steps[step_num].img.as_ref().unwrap()
	}

    pub fn check_if_can_start_processing(&self, step_num: usize) -> StartProcResult {
        if self.initial_img.is_none() {
            StartProcResult::NoInitialImg
        } else if step_num > 0 && self.proc_steps[step_num - 1].img.is_none() {
            StartProcResult::NoPrevStepImg
        } else {
            StartProcResult::CanStart
        }
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
}


pub enum StartProcResult { NoInitialImg, NoPrevStepImg, CanStart }
pub enum StartResultsSavingResult { NoSteps, NotAllStepsHaveResult, CanStart }
