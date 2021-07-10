use fltk::prelude::ImageExt;

use crate::{img::{Img, PixelsArea}, my_err::MyError, processing::TaskMsg};
use super::{ProgressProvider, guarded::Guarded};


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

        let mut prog_prov = ProgressProvider::new(
            &guarded.tx_notify, 
            &guarded.rx_halt);

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
        let img_result = match step.filter.filter(&img_to_process, &mut prog_prov) {
            Ok(img) => {
                // assert!(prog_prov.all_actions_completed());
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