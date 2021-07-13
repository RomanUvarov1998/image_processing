use fltk::{image::RgbImage};
use crate::{filter::FilterBase, img::Img, my_err::MyError};
use super::{TaskMsg, progress_provider::HaltMessage};

pub struct Guarded {
	tx_notify: std::sync::mpsc::Sender<TaskMsg>,
	rx_halt: std::sync::mpsc::Receiver<HaltMessage>,
    task: Option<tasks::TaskBase>,
    task_result: Option<Result<(), MyError>>,
    initial_step: Step,
    proc_steps: Vec<ProcStep>,
}

impl Guarded {
	pub fn new(progress_tx: std::sync::mpsc::Sender<TaskMsg>, rx_halt: std::sync::mpsc::Receiver<HaltMessage>) -> Self {
		Guarded {
			tx_notify: progress_tx, rx_halt,
			task: None, task_result: None,
			initial_step: Step { img: None },
			proc_steps: Vec::new()
		}
	}

	pub fn has_task_to_do(&self) -> bool {
        self.task.is_some() && self.task_result.is_none()
	}

	pub fn do_task_and_save_result(&mut self) {
		// make the halt message buffer clean
		while let Ok(_) = self.rx_halt.try_recv() {}

        let mut task: tasks::TaskBase = self.task.take().unwrap();
        self.task_result = Some(task.complete(self));

        self.tx_notify.send( TaskMsg::Finished ).unwrap();
	}
    
    pub fn start_task(&mut self, task: tasks::TaskBase) {
        assert!(self.task.is_none());
        self.task = Some(task);
	}

    pub fn get_task_result(&mut self) -> Result<(), MyError> {
        self.task_result.take().unwrap()
    }


	pub fn get_initial_img(&self) -> &Img {
        self.initial_step.img.as_ref().unwrap()
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
	
	pub fn get_step_img(&self, step_num: usize) -> &Img {
        self.proc_steps[step_num].img.as_ref().unwrap()
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


pub mod tasks {
    use fltk::prelude::ImageExt;
    use crate::{img::{Img, PixelsArea}, my_err::MyError, processing::TaskMsg};
    use super::Guarded;
    use super::super::ProgressProvider;


    pub trait Task {
        fn complete(&mut self, guarded: &mut Guarded) -> Result<(), MyError>;
    }

    pub type TaskBase = Box<dyn Task + Send>;


    pub struct ProcTask {
        step_num: usize, 
        crop_area: Option<PixelsArea>,
    }

    impl ProcTask {
        pub fn new(step_num: usize, crop_area: Option<PixelsArea>) -> TaskBase {
            let task = ProcTask { step_num, crop_area };
            Box::new(task) as TaskBase
        }
    }

    impl Task for ProcTask {
        fn complete(&mut self, guarded: &mut Guarded) -> Result<(), MyError> {
            for step in &mut guarded.proc_steps[self.step_num + 1..] {
                if let Some(_) = step.img {
                    step.img = None;
                }
            }

            let mut img_to_process: &Img = if self.step_num == 0 {
                guarded.get_initial_img()
            } else {
                guarded.get_step_img(self.step_num - 1)
            };

            let cropped_copy: Img;
            if let Some(crop_area) = self.crop_area {
                cropped_copy = img_to_process.get_cropped_copy(crop_area);
                img_to_process = &cropped_copy;
            }

            let step = &guarded.proc_steps[self.step_num];

            let mut prog_prov = ProgressProvider::new(
                &guarded.tx_notify, 
                &guarded.rx_halt,
                step.filter.get_steps_num(&img_to_process));

            let img_result = match step.filter.filter(&img_to_process, &mut prog_prov) {
                Ok(img) => {
                    prog_prov.assert_all_actions_completed();
                    Some(img)
                },
                Err(_halted) => None
            };

            guarded.proc_steps[self.step_num].img = img_result;

            Ok(())
        }
    }


    #[derive(Debug)]
    pub struct ExportTask {
        dir_path: String,
    }

    impl ExportTask {
        pub fn new(dir_path: String) -> TaskBase {
            let task = ExportTask { dir_path };
            Box::new(task) as TaskBase
        }
    }

    impl Task for ExportTask {
        fn complete(&mut self, guarded: &mut Guarded) -> Result<(), MyError> {
            if let Err(err) = std::fs::create_dir(&self.dir_path) {
                return Err(MyError::new(err.to_string()));
            };

            for step_num in 0..guarded.proc_steps.len() {
                let mut file_path = self.dir_path.clone();
                file_path.push_str(&format!("/{}.jpg", step_num + 1));
                
                let step = &guarded.proc_steps[step_num];
                step.img.as_ref().unwrap().try_save(&file_path)?;
                
                let percents = step_num * 100 / guarded.proc_steps.len();
                guarded.tx_notify.send( TaskMsg::Progress { percents } ).unwrap();
            }

            Ok(())
        }
    }


    #[derive(Debug)]
    pub struct ImportTask {
        path: String,
    }

    impl ImportTask {
        pub fn new(path: String) -> TaskBase {
            let task = ImportTask { path };
            Box::new(task) as TaskBase
        }
    }

    impl Task for ImportTask {
        fn complete(&mut self, guarded: &mut Guarded) -> Result<(), MyError> {
            let sh_im = fltk::image::SharedImage::load(&self.path)?;

            guarded.tx_notify.send(TaskMsg::Progress { percents: 50 } ).unwrap();

            if sh_im.w() < 0 { 
                return Err(MyError::new("Ширина загруженного изображения < 0".to_string())); 
            }
            if sh_im.h() < 0 { 
                return Err(MyError::new("Высота загруженного изображения < 0".to_string())); 
            }

            let img = Img::from(sh_im);

            guarded.initial_step.img = Some(img);

            Ok(())
        }
    }

    const FILTER_SAVE_SEPARATOR: &'static str = "||";
    pub const PROJECT_EXT: &'static str = "ps";

    #[derive(Debug)]
    pub struct SaveProjectTask {
        path: String,
    }

    impl SaveProjectTask {
        pub fn new(path: String) -> TaskBase {
            let task = SaveProjectTask { path };
            Box::new(task) as TaskBase
        }
    }

    impl Task for SaveProjectTask {
        fn complete(&mut self, guarded: &mut Guarded) -> Result<(), MyError> {
            let mut file = match std::fs::File::create(&self.path) {
                Ok(f) => f,
                Err(err) => { return Err(MyError::new(err.to_string())); }
            };

            let mut file_content = String::new();

            for step_num in 0..guarded.proc_steps.len() {
                let filter_save_name: String = guarded.get_filter_save_name(step_num);

                file_content.push_str(&filter_save_name);
                file_content.push_str("\n");

                if let Some(params_str) = guarded.get_filter_params_as_str(step_num) {
                    file_content.push_str(&params_str);
                }
                file_content.push_str("\n");

                if step_num < guarded.proc_steps.len() - 1 {
                    file_content.push_str(FILTER_SAVE_SEPARATOR);
                    file_content.push_str("\n");
                }
            }

            use std::io::Write;
            file.write_all(&file_content.as_bytes())?;
            file.sync_all()?;

            Ok(())
        }
    }


    #[derive(Debug)]
    pub struct LoadProjectTask {
        path: String,
    }

    impl LoadProjectTask {
        pub fn new(path: String) -> TaskBase {
            let task = LoadProjectTask { path };
            Box::new(task) as TaskBase
        }

        fn remove_all_steps(guarded: &mut Guarded) {
            guarded.proc_steps.clear();
        }
    }

    impl Task for LoadProjectTask {
    fn complete(&mut self, guarded: &mut Guarded) -> Result<(), MyError> {
        Self::remove_all_steps(guarded);

        let mut file = match std::fs::File::open(&self.path) {
            Ok(f) => f,
            Err(err) => { return Err(MyError::new(format!("Ошибка при открытии файла проекта: {}", err.to_string()))); },
        };

        let mut file_content = String::new();
        use std::io::Read;
        if let Err(err) = file.read_to_string(&mut file_content) {
            return Err(MyError::new(format!("Ошибка при чтении файла проекта: {}", err.to_string())));
        };

        let mut filters_iter = crate::utils::TextBlocksIter::new(
            &file_content, FILTER_SAVE_SEPARATOR);

        guarded.proc_steps.reserve(filters_iter.len());

        let steps_count = filters_iter.len();

        for (step_num, filter_str) in filters_iter.iter().enumerate() {
            let mut lines_iter = crate::utils::LinesIter::new(filter_str);
            let filter_name = lines_iter.next_or_empty().to_string();
            let filter_content = lines_iter.all_left(true);

            match crate::filter::try_parce_filter(&filter_name, &filter_content) {
                Ok(filter) => {
                    guarded.proc_steps.push(crate::processing::guarded::ProcStep { img: None, filter } );
                },
                Err(err) => {
                    let err_msg = format!(
                        "Ошибка формата при чтении фильтра '{}': '{}'", 
                        filter_name, err.to_string());
                    Self::remove_all_steps(guarded);
                    return Err(MyError::new(err_msg));
                },
            }

            let percents = step_num * 100 / steps_count;
            guarded.tx_notify.send( TaskMsg::Progress { percents } ).unwrap();
        }

        Ok(())
    }
}
}