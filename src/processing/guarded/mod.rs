use core::panic;

use super::ExecutorHandle;
use crate::{
    img::{filter::FilterBase, Img, PixelsArea},
    my_err::MyError,
    processing::task_info_channel::{TaskState, TaskStop},
};
use fltk::image::RgbImage;
use proc_step::ProcStep;

mod proc_step;

pub struct Guarded {
    executor_handle: ExecutorHandle,
    task_setup: Option<TaskSetup>,
    initial_img: Option<Img>,
    proc_steps: Vec<ProcStep>,
}

impl Guarded {
    pub fn new(executor_handle: ExecutorHandle) -> Self {
        Guarded {
            executor_handle,
            task_setup: None,
            initial_img: None,
            proc_steps: Vec::new(),
        }
    }

    pub fn has_task_to_do(&self) -> bool {
        self.task_setup.is_some()
    }

    pub fn do_task_and_save_result(&mut self) {
        print!("started ");

        let task_setup: TaskSetup = self.task_setup.take().expect("No task was set up!");

        let result: Result<(), TaskStop> = match &task_setup {
            TaskSetup::ProcessStep {
                step_num,
                crop_area,
            } => Self::process_step(
                &mut self.executor_handle,
                &self.initial_img,
                &mut self.proc_steps,
                *step_num,
                *crop_area,
            ),
            TaskSetup::Export { ref dir_path } => {
                Self::export_results(&mut self.executor_handle, &self.proc_steps, dir_path)
            }
            TaskSetup::Import { file_path } => {
                Self::import(&mut self.executor_handle, &mut self.initial_img, file_path)
            }
            TaskSetup::SaveProject { file_path } => {
                Self::save_project(&mut self.executor_handle, &self.proc_steps, file_path)
            }
            TaskSetup::LoadProject { file_path } => {
                Self::load_project(&mut self.executor_handle, &mut self.proc_steps, file_path)
            }
        };

        println!("result {:?}", result);

        match self.executor_handle.get_task_state() {
            TaskState::Empty => panic!("No task detected"),
            TaskState::InProgress { .. } => panic!("Task wasn't completed "),
            TaskState::Finished { .. } => {}
        }

        println!("{:?}", self.task_setup);
    }

    pub fn start_task(&mut self, setup: TaskSetup) {
        assert!(self.task_setup.is_none());
        self.task_setup = Some(setup);
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
        self.proc_steps.push(ProcStep { img: None, filter });
    }

    pub fn edit_step(
        &mut self,
        step_num: usize,
        mut action: impl FnMut(&mut FilterBase) -> bool,
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
        executor_handle: &mut ExecutorHandle,
        initial_img: &Option<Img>,
        proc_steps: &mut Vec<ProcStep>,
        step_num: usize,
        crop_area: Option<PixelsArea>,
    ) -> Result<(), TaskStop> {
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

        let img_result = match step.filter.process(&img_to_process, executor_handle) {
            Ok(img) => Some(img),
            Err(_halted) => None,
        };

        proc_steps[step_num].img = img_result;

        Ok(())
    }

    fn export_results(
        executor_handle: &mut ExecutorHandle,
        proc_steps: &Vec<ProcStep>,
        dir_path: &str,
    ) -> Result<(), TaskStop> {
        executor_handle.reset(1 + proc_steps.len());

        std::fs::create_dir(&dir_path)?;

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
        executor_handle: &mut ExecutorHandle,
        initial_img: &mut Option<Img>,
        file_path: &str,
    ) -> Result<(), TaskStop> {
        executor_handle.reset(2);

        println!("loadeding...");
        let sh_im = fltk::image::SharedImage::load(file_path)?;
        println!("loaded");

        executor_handle.complete_action()?;

        use fltk::prelude::ImageExt;
        if sh_im.w() < 0 {
            return Err(MyError::new("???????????? ???????????????????????? ?????????????????????? < 0".to_string()).into());
        }
        if sh_im.h() < 0 {
            return Err(MyError::new("???????????? ???????????????????????? ?????????????????????? < 0".to_string()).into());
        }

        let img = Img::from_pixels(
            sh_im.w() as usize,
            sh_im.h() as usize,
            sh_im.depth(),
            sh_im.to_rgb_data(),
        );

        initial_img.replace(img);

        executor_handle.complete_action()?;

        Ok(())
    }

    fn save_project(
        executor_handle: &mut ExecutorHandle,
        proc_steps: &Vec<ProcStep>,
        file_path: &str,
    ) -> Result<(), TaskStop> {
        executor_handle.reset(1 + proc_steps.len() + 1);

        let mut file = std::fs::File::create(file_path)?;

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
        executor_handle: &mut ExecutorHandle,
        proc_steps: &mut Vec<ProcStep>,
        file_path: &str,
    ) -> Result<(), TaskStop> {
        executor_handle.reset(1 + 1 + 1);

        proc_steps.clear();

        executor_handle.complete_action()?;

        let mut file = std::fs::File::open(file_path)?;

        let mut file_content = String::new();
        use std::io::Read;
        file.read_to_string(&mut file_content)?;

        executor_handle.complete_action()?;

        let mut filters_iter =
            crate::utils::TextBlocksIter::new(&file_content, FILTER_SAVE_SEPARATOR);

        proc_steps.reserve(filters_iter.len());

        for filter_str in filters_iter.iter() {
            let mut lines_iter = crate::utils::LinesIter::new(filter_str);
            let filter_name = lines_iter.next_or_empty().to_string();
            let filter_content = lines_iter.all_left(true);

            use crate::img::filter::try_parce_filter;
            let filter = try_parce_filter(&filter_name, &filter_content)?;
            proc_steps.push(ProcStep { img: None, filter });
        }

        executor_handle.complete_action()?;

        Ok(())
    }
}

const FILTER_SAVE_SEPARATOR: &'static str = "||";
pub const PROJECT_EXT: &'static str = "ps";

#[derive(Debug)]
pub enum TaskSetup {
    ProcessStep {
        step_num: usize,
        crop_area: Option<PixelsArea>,
    },
    Export {
        dir_path: String,
    },
    Import {
        file_path: String,
    },
    SaveProject {
        file_path: String,
    },
    LoadProject {
        file_path: String,
    },
}

pub enum StartProcResult {
    NoInitialImg,
    NoPrevStepImg,
    CanStart,
}
pub enum StartResultsSavingResult {
    NoSteps,
    NotAllStepsHaveResult,
    CanStart,
}
