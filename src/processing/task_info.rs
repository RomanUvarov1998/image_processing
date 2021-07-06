use fltk::image::RgbImage;
use crate::{img::PixelsArea, my_err::MyError};
use super::BWError;


#[derive(Debug)]
pub enum TaskInfo<T, R> {
    Setup ( Option<T> ),
    Result ( Option<Result<R, BWError>> )
}

impl<T, R> TaskInfo<T, R> where T: std::fmt::Debug, R: std::fmt::Debug {
    pub fn setup(task: T) -> Self {
        TaskInfo::Setup( Some(task) )
    }
    
	pub fn result(result: Result<R, BWError>) -> Self {
        TaskInfo::Result( Some(result) )
    }

    pub fn is_setup(&self) -> bool {
        match self {
            TaskInfo::Setup(_) => true,
            TaskInfo::Result(_) => false,
        }
    }

	pub fn result_was_taken(&self) -> bool {
		match self {
			TaskInfo::Setup(_) => false,
			TaskInfo::Result(result_op) => result_op.is_none(),
		}
	}

    pub fn take_setup(&mut self) -> Result<T, BWError> {
        if let TaskInfo::Setup(task_op) = self {
            if let Some(task) = task_op.take() {
                return Ok(task);
            }
        }

        Err ( BWError::TaskIsEmpty )
    }

    pub fn take_result(&mut self) -> Result<R, BWError> {
        if let TaskInfo::Result(result_op) = self {
            if let Some(result) = result_op.take() {
                return result;
            }
        }

        Err ( BWError::TaskResultIsEmpty )
    }
}


#[derive(Debug)]
pub struct ProcSetup { 
    pub step_num: usize, 
    pub crop_area: Option<PixelsArea> 
}


#[derive(Debug)]
pub struct ProcResult { 
    pub img: Option<RgbImage>, 
    pub it_is_the_last_step: bool,
    pub processing_was_halted: bool
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


#[derive(Debug)]
pub struct ExportSetup {
    pub dir_path: String
}


#[derive(Debug)]
pub struct ExportResult {
    pub result: Result<(), MyError>
}