use std::fs;
use fltk::{image::RgbImage, prelude::ImageExt};
use crate::{img::{Img, PixelsArea}, message::{Export, Msg, Proc, Project}, my_err::MyError};
use super::{BWError, FilterBase, ProgressProvider, progress_provider::HaltMessage, task_info::*};

pub struct Guarded {
    exch: ThreadExchange,
    initial_step: Step,
    proc_steps: Vec<ProcStep>,
}

impl Guarded {
	pub fn new() -> Self {
		Guarded {
			exch: ThreadExchange::Empty,
			initial_step: Step { img: None },
			proc_steps: Vec::new()
		}
	}

	pub fn has_task_to_do(&self) -> bool {
		match self.exch {
			ThreadExchange::Empty => false,
			ThreadExchange::Proc(ref task_info) => task_info.is_setup(),
			ThreadExchange::Export(ref task_info) => task_info.is_setup(),
		}
	}

	pub fn do_task_and_save_result(
		&mut self,
        progress_tx: &fltk::app::Sender<Msg>,
        rx_halt: &std::sync::mpsc::Receiver<HaltMessage>
	) {
		self.exch = match self.exch {
			ThreadExchange::Empty => ThreadExchange::Empty,
			ThreadExchange::Proc(ref mut task_info_op) => {
				let proc_result = Self::try_process(
					&self.initial_step,
					&mut self.proc_steps,
					task_info_op, 
					progress_tx,
					rx_halt);

				ThreadExchange::Proc( TaskInfo::result(proc_result) )
			},
			ThreadExchange::Export(ref mut task_info_op) => {
				let export_result = Self::try_export(
					&self.proc_steps, 
					task_info_op, 
					progress_tx);

				ThreadExchange::Export( TaskInfo::result(export_result) )
			},
		}
	}

	fn try_process(
		initial_step: &Step,
		proc_steps: &mut Vec<ProcStep>,
		task_info_op: &mut TaskInfo<ProcSetup, ProcResult>,
        progress_tx: &fltk::app::Sender<Msg>,
        rx_halt: &std::sync::mpsc::Receiver<HaltMessage>
	) -> Result<ProcResult, BWError> {
		let setup = task_info_op.take_setup()?;
		let step_num = setup.step_num;

		let mut prog_prov = ProgressProvider::new(
			&progress_tx, 
			rx_halt, 
			step_num,
			proc_steps.len()
		);

		// leave the message buffer clean
		while let Ok(_) = rx_halt.try_recv() {}
		
		let mut img_to_process: &Img = if step_num == 0 {
			Self::try_get_initial_img(initial_step)?
		} else {
			Self::try_get_prev_step_img(proc_steps, step_num)?
		};

		let cropped_copy: Img;
		if let Some(crop_area) = setup.crop_area {
			cropped_copy = img_to_process.get_cropped_copy(crop_area);
			img_to_process = &cropped_copy;
		}

		let step = &proc_steps[step_num];
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
			it_is_the_last_step: setup.step_num == proc_steps.len() - 1,
			processing_was_halted: img_result.is_none(),
		};

		proc_steps[setup.step_num].step.img = img_result;

		progress_tx.send(Msg::Proc(Proc::CompletedStep { step_num: setup.step_num }));

		Ok(task_result)
	}

	fn try_export(
		proc_steps: &Vec<ProcStep>,
		task_info_op: &mut TaskInfo<ExportSetup, ExportResult>,
        progress_tx: &fltk::app::Sender<Msg>
	) -> Result<ExportResult, BWError> {
		let setup = task_info_op.take_setup()?;

		if let Err(err) = fs::create_dir(&setup.dir_path) {
			progress_tx.send(Msg::Project( Project::Export ( Export::Completed ) ) );
			let io_err = MyError::new(err.to_string());
			return Ok( ExportResult { result: Err(io_err) } ); 
		};

		for step_num in 0..proc_steps.len() {
			let mut file_path = setup.dir_path.clone();
			file_path.push_str(&format!("/{}.jpg", step_num + 1));
			
			let step_img = Self::try_get_step_img(proc_steps, step_num)?;
			if let Err(err) = step_img.try_save(&file_path) {
				progress_tx.send(Msg::Project( Project::Export ( Export::Completed ) ) );
				let io_err = MyError::new(err.to_string());
				return Ok( ExportResult { result: Err(io_err) } ); 
			}

			let percents = step_num * 100 / proc_steps.len();
			progress_tx.send(Msg::Project( Project::Export ( Export::Progress { percents } ) ) );
		}

		progress_tx.send(Msg::Project( Project::Export ( Export::Completed ) ) );
		Ok( ExportResult { result: Ok(()) } )
	}

	pub fn try_load_initial_img(&mut self, path: &str) -> Result<(), MyError> {
        let sh_im = fltk::image::SharedImage::load(path)?;

        if sh_im.w() < 0 { return Err(MyError::new("Ширина загруженного изображения < 0".to_string())); }
        if sh_im.h() < 0 { return Err(MyError::new("Высота загруженного изображения < 0".to_string())); }

        let img = Img::from(sh_im);

        self.initial_step.img = Some(img);

        Ok(())
	}

    pub fn has_initial_img(&self) -> bool {
        self.initial_step.img.is_some()
    }

    pub fn get_init_img_drawable(&self) -> Result<RgbImage, BWError> {
        match self.initial_step.img.as_ref() {
            Some(img) => Ok(img.get_drawable_copy()),
            None => Err(BWError::NoInitialImage),
        }
    }
    
    pub fn get_init_img_descr(&self) -> Result<String, BWError> {
        match self.initial_step.img.as_ref() {
            Some(img) => Ok(img.get_description()),
            None => Err(BWError::NoInitialImage),
        }
    }

	
    pub fn add_step(&mut self, filter: FilterBase) {
        self.proc_steps.push( 
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
    ) -> Result<bool, BWError> {
        Self::check_if_step_num_exceeds_bounds(&self.proc_steps, step_num)?;
        let action_result = action(&mut self.proc_steps[step_num].filter);
        Ok(action_result)
    }

    pub fn remove_step(&mut self, step_num: usize) -> Result<(), BWError> {
        Self::check_if_step_num_exceeds_bounds(&self.proc_steps, step_num)?;
        self.proc_steps.remove(step_num);
        Ok(())
    }

	pub fn swap_steps(&mut self, step_num1: usize, step_num2: usize) -> Result<(), BWError> {
        Self::check_if_step_num_exceeds_bounds(&self.proc_steps, step_num1)?;
        Self::check_if_step_num_exceeds_bounds(&self.proc_steps, step_num2)?; 

		self.proc_steps.swap(step_num1, step_num2);

		Ok(())
    }


    pub fn check_if_can_start_processing(&self, step_num: usize) -> StartProcResult {
        if self.initial_step.img.is_none() {
            StartProcResult::NoInitialImg
        } else if step_num > 0 && self.proc_steps[step_num - 1].step.img.is_none() {
            StartProcResult::NoPrevStepImg
        } else {
            StartProcResult::CanStart
        }
    }

    pub fn start_processing(
		&mut self, 
		step_num: usize, 
		crop_area: Option<PixelsArea>
	) -> Result<(), BWError> {
		if self.can_start_task() {
			for step in &mut self.proc_steps[step_num..] {
				if let Some(_) = step.step.img {
					step.step.img = None;
				}
			}

			self.exch = ThreadExchange::Proc( TaskInfo::setup(ProcSetup { step_num, crop_area }));
			Ok(())
		} else {
			Err( BWError::ThreadExchangeNotEmpty )
		}
    }

    pub fn get_proc_result(&mut self) -> Result<ProcResult, BWError> {
        match self.exch {
            ThreadExchange::Proc(ref mut task_info) => 
				Ok(task_info.take_result()?),
            _ => Err(BWError::NotFoundExpectedTask),
        }
    }

    pub fn get_step_descr(&self, step_num: usize) -> Result<String, BWError> {
        Self::check_if_step_num_exceeds_bounds(&self.proc_steps, step_num)?;
        let descr = self.proc_steps[step_num].get_description();
        Ok(descr)
    }


    pub fn get_filter_params_as_str(&self, step_num: usize) -> Result<Option<String>, BWError> {
        Self::check_if_step_num_exceeds_bounds(&self.proc_steps, step_num)?;
        let params_str = self.proc_steps[step_num].filter.params_to_string();
        Ok(params_str)
    }
    
    pub fn get_filter_save_name(&self, step_num: usize) -> Result<String, BWError> {
        Self::check_if_step_num_exceeds_bounds(&self.proc_steps, step_num)?;
        let save_name = self.proc_steps[step_num].filter.get_save_name();
        Ok(save_name)
    }


    pub fn check_if_can_export(&self) -> StartResultsSavingResult {
        if self.proc_steps.len() == 0 {
            StartResultsSavingResult::NoSteps
        } else if self.proc_steps.iter().any(|s| s.step.img.is_none()) {
            StartResultsSavingResult::NotAllStepsHaveResult
        } else {
            StartResultsSavingResult::CanStart
        }
    }

    pub fn start_export(&mut self, dir_path: String) -> Result<(), BWError> {
		if self.can_start_task() {
			self.exch = ThreadExchange::Export( TaskInfo::setup(ExportSetup { dir_path }) );
			Ok(())
		} else {
			Err( BWError::ThreadExchangeNotEmpty )
		}
    }

    pub fn get_export_result(&mut self) -> Result<ExportResult, BWError> {
        match self.exch {
            ThreadExchange::Export(ref mut task) => 
				Ok(task.take_result()?),
            _ => Err(BWError::NotFoundExpectedTask)
        }
    }


	fn try_get_initial_img(initial_step: &Step) -> Result<&Img, BWError> {
		match initial_step.img {
			Some(ref img) => Ok(img),
			None => Err(BWError::NoInitialImage),
		}
	}
	fn try_get_prev_step_img(proc_steps: &Vec<ProcStep>, step_num: usize) -> Result<&Img, BWError> {
		let prev_step_num = step_num - 1;
		Self::check_if_step_num_exceeds_bounds(proc_steps, prev_step_num)?;
		match proc_steps[prev_step_num].step.img {
			Some(ref img) => Ok(img),
			None => Err(BWError::NoPrevStepImg),
		}
	}
	fn try_get_step_img(proc_steps: &Vec<ProcStep>, step_num: usize) -> Result<&Img, BWError> {
		Self::check_if_step_num_exceeds_bounds(proc_steps, step_num)?;
		match proc_steps[step_num].step.img {
			Some(ref img) => Ok(img),
			None => Err(BWError::NoStepImg),
		}
	}

	fn can_start_task(&self) -> bool {
		match self.exch {
			ThreadExchange::Empty => true,
			ThreadExchange::Proc(ref task_info) => task_info.result_was_taken(),
			ThreadExchange::Export(ref task_info) => task_info.result_was_taken(),
		}
	}

    fn check_if_step_num_exceeds_bounds(proc_steps: &Vec<ProcStep>, step_num: usize) -> Result<(), BWError> {
        match proc_steps.get(step_num) {
            Some(_) => Ok(()),
            None => Err(BWError::StepNumExceedsBounds),
        }
    }
}


struct Step { img: Option<Img> }


pub struct ProcStep {
    step: Step,
    filter: FilterBase
}

impl ProcStep {
    pub fn get_description(&self) -> String {
        let filter_descr = self.filter.get_description();

        let img_descr = match self.step.img {
            Some(ref img) => img.get_description(),
            None => String::new(),
        };

        format!("{} {}", &filter_descr, &img_descr)
    }
}

#[derive(Debug)]
enum ThreadExchange {
    Empty,
    Proc ( TaskInfo<ProcSetup, ProcResult> ),
    Export ( TaskInfo<ExportSetup, ExportResult> ),
}


pub enum StartProcResult { NoInitialImg, NoPrevStepImg, CanStart }


pub enum StartResultsSavingResult { NoSteps, NotAllStepsHaveResult, CanStart }