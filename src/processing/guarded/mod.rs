use crate::{img::{Img, PixelsArea, filter::FilterBase}, my_err::MyError};
use fltk::image::RgbImage;
use proc_step::ProcStep;
use super::ExecutorHandle;

mod proc_step;


pub struct Guarded {
	executor_handle: ExecutorHandle,
    task: Option<Task>,
    initial_img: Option<Img>,
    proc_steps: Vec<ProcStep>,
}

impl Guarded {
	pub fn new(executor_handle: ExecutorHandle) -> Self {
		Guarded {
			executor_handle,
			task: None,
			initial_img: None,
			proc_steps: Vec::new()
		}
	}

	pub fn has_task_to_do(&self) -> bool {
        match self.task {
            Some(ref task) => task.result.is_none(),
            None => false
        }
	}

	pub fn do_task_and_save_result(&mut self) {
        print!("started ");

        let task: &Task = self.task.as_ref().expect("No task was set up!");
        let result: Result<(), MyError> = match &task.setup {
            TaskSetup::ProcessStep { step_num, crop_area } => Self::process_step(
                &self.executor_handle,
                &self.initial_img, 
                &mut self.proc_steps, 
                *step_num, *crop_area),
            TaskSetup::Export { ref dir_path } => Self::export_results(
                &self.executor_handle,
                &self.proc_steps,
                dir_path),
            TaskSetup::Import { file_path } => Self::import(
                &self.executor_handle,
                &mut self.initial_img, 
                file_path),
            TaskSetup::SaveProject { file_path } => Self::save_project(
                &self.executor_handle,
                &self.proc_steps,
                file_path),
            TaskSetup::LoadProject { file_path } => Self::load_project(
                &self.executor_handle,
                &mut self.proc_steps, 
                file_path),
        };
        
        let task: &mut Task = self.task.as_mut().expect("No task was set up!");
        task.result = Some(result);

        if !self.executor_handle.task_is_halted() {
            println!("completed ");
            self.executor_handle.assert_all_actions_completed();
        } else {
            self.task.take();
            println!("halted ");
        }

        println!("{:?}", self.task);
	}
    
    pub fn start_task(&mut self, setup: TaskSetup) {
        assert!(self.task.is_none());

        let task = Task { setup, result: None };
        
        if !self.executor_handle.task_is_halted() {
            self.executor_handle.assert_all_actions_completed();
        }
        
        self.task = Some(task);            
	}

    pub fn get_task_result(&mut self) -> Result<(), MyError> {
        let mut task: Task = self.task.take().unwrap();
        task.result.take().unwrap()
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


    fn process_step(
        executor_handle: &ExecutorHandle, 
        initial_img: &Option<Img>, 
        proc_steps: &mut Vec<ProcStep>, 
        step_num: usize, crop_area: Option<PixelsArea>
    ) -> Result<(), MyError> {
        for step in &mut proc_steps[step_num + 1..] {
            if let Some(_) = step.img {
                step.img = None;
            }
        }

        let mut img_to_process: &Img = if step_num == 0 {
            initial_img.as_ref().unwrap()
        } else {
            proc_steps[step_num - 1].img.as_ref().unwrap()
        };

        let cropped_copy: Img;
        if let Some(crop_area) = crop_area {
            cropped_copy = img_to_process.get_cropped_copy(crop_area);
            img_to_process = &cropped_copy;
        }

        let step = &proc_steps[step_num];

        executor_handle.reset(step.filter.get_steps_num(&img_to_process));

        let img_result = match step.filter.process(&img_to_process, &executor_handle) {
            Ok(img) => Some(img),
            Err(_halted) => None
        };

        proc_steps[step_num].img = img_result;

        Ok(())
    }

    fn export_results(
        executor_handle: &ExecutorHandle, 
        proc_steps: &Vec<ProcStep>, 
        dir_path: &str
    ) -> Result<(), MyError> {
        executor_handle.reset(1 + proc_steps.len());

        if let Err(err) = std::fs::create_dir(&dir_path) {
            return Err(MyError::new(err.to_string()));
        };

        executor_handle.complete_action()?;

        for step_num in 0..proc_steps.len() {
            let mut file_path = String::from(dir_path);
            file_path.push_str(&format!("/{}.jpg", step_num + 1));
            
            let step = &proc_steps[step_num];
            step.img.as_ref().unwrap().try_save(&file_path)?;
            
            executor_handle.complete_action()?;
        }

        Ok(())
    }

    fn import(
        executor_handle: &ExecutorHandle, 
        initial_img: &mut Option<Img>, 
        file_path: &str
    )-> Result<(), MyError> {
        executor_handle.reset(2);

        let sh_im = fltk::image::SharedImage::load(file_path)?;

        executor_handle.complete_action()?;

        use fltk::prelude::ImageExt;
        if sh_im.w() < 0 { 
            return Err(MyError::new("Ширина загруженного изображения < 0".to_string())); 
        }
        if sh_im.h() < 0 { 
            return Err(MyError::new("Высота загруженного изображения < 0".to_string())); 
        }

        let img = Img::from(sh_im);

        initial_img.replace(img);

        executor_handle.complete_action()?;

        Ok(())
    }

    fn save_project(
        executor_handle: &ExecutorHandle, 
        proc_steps: &Vec<ProcStep>, 
        file_path: &str
    ) -> Result<(), MyError> {
        executor_handle.reset(1 + proc_steps.len() + 1);

        let mut file = match std::fs::File::create(file_path) {
            Ok(f) => f,
            Err(err) => { return Err(MyError::new(err.to_string())); }
        };

        executor_handle.complete_action()?;

        let mut file_content = String::new();

        for step_num in 0..proc_steps.len() {
            let filter_save_name: String = proc_steps[step_num].filter.get_save_name();

            file_content.push_str(&filter_save_name);
            file_content.push_str("\n");

            if let Some(params_str) = proc_steps[step_num].filter.params_to_string() {
                file_content.push_str(&params_str);
            }
            file_content.push_str("\n");

            if step_num < proc_steps.len() - 1 {
                file_content.push_str(FILTER_SAVE_SEPARATOR);
                file_content.push_str("\n");
            }

            executor_handle.complete_action()?;
        }

        use std::io::Write;
        file.write_all(&file_content.as_bytes())?;
        file.sync_all()?;

        executor_handle.complete_action()?;

        Ok(())
    }

    fn load_project(
        executor_handle: &ExecutorHandle, 
        proc_steps: &mut Vec<ProcStep>, 
        file_path: &str
    ) -> Result<(), MyError> {
        executor_handle.reset(1 + 1 + 1);

        proc_steps.clear();

        executor_handle.complete_action()?;

        let mut file = match std::fs::File::open(file_path) {
            Ok(f) => f,
            Err(err) => { return Err(MyError::new(format!("Ошибка при открытии файла проекта: {}", err.to_string()))); },
        };

        let mut file_content = String::new();
        use std::io::Read;
        if let Err(err) = file.read_to_string(&mut file_content) {
            return Err(MyError::new(format!("Ошибка при чтении файла проекта: {}", err.to_string())));
        };

        executor_handle.complete_action()?;

        let mut filters_iter = crate::utils::TextBlocksIter::new(
            &file_content, FILTER_SAVE_SEPARATOR);

        proc_steps.reserve(filters_iter.len());

        for filter_str in filters_iter.iter() {
            let mut lines_iter = crate::utils::LinesIter::new(filter_str);
            let filter_name = lines_iter.next_or_empty().to_string();
            let filter_content = lines_iter.all_left(true);

            match crate::img::filter::try_parce_filter(&filter_name, &filter_content) {
                Ok(filter) => {
                    proc_steps.push(ProcStep { img: None, filter } );
                },
                Err(err) => {
                    let err_msg = format!(
                        "Ошибка формата при чтении фильтра '{}': '{}'", 
                        filter_name, err.to_string());
                    proc_steps.clear();
                    return Err(MyError::new(err_msg));
                },
            }
        }

        executor_handle.complete_action()?;

        Ok(())
    }

}

const FILTER_SAVE_SEPARATOR: &'static str = "||";
pub const PROJECT_EXT: &'static str = "ps";

#[derive(Debug)]
pub enum TaskSetup {
    ProcessStep { step_num: usize, crop_area: Option<PixelsArea> },
    Export { dir_path: String },
    Import { file_path: String },
    SaveProject { file_path: String },
    LoadProject { file_path: String },
}

#[derive(Debug)]
struct Task {
    setup: TaskSetup,
    result: Option<Result<(), MyError>>,
}


pub enum StartProcResult { NoInitialImg, NoPrevStepImg, CanStart }
pub enum StartResultsSavingResult { NoSteps, NotAllStepsHaveResult, CanStart }
