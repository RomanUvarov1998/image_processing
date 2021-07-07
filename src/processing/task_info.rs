use crate::{img::{Img, PixelsArea}, message::TaskMsg, my_err::MyError};
use super::{ProgressProvider, guarded::Guarded};


pub trait Task {
    fn is_completed(&self) -> bool;
    fn complete(&mut self, guarded: &mut Guarded);
    fn take_result(&mut self) -> Result<(), MyError>;
}

pub type TaskBase = Box<dyn Task + Send>;


pub struct ProcTask {
    step_num: usize, 
    crop_area: Option<PixelsArea>,
    result: Option<Result<(), MyError>>
}

impl ProcTask {
    pub fn new(step_num: usize, crop_area: Option<PixelsArea>) -> TaskBase {
        let task = ProcTask {
            step_num,
            crop_area,
            result: None
        };
        Box::new(task) as TaskBase
    }

    fn process(&self, guarded: &mut Guarded) -> Result<(), MyError> {
		for step in &mut guarded.proc_steps[self.step_num + 1..] {
			if let Some(_) = step.img {
				step.img = None;
			}
		}

        let mut prog_prov = ProgressProvider::new(
            &guarded.tx_notify, 
            &guarded.rx_halt);

        let mut img_to_process: &Img = if self.step_num == 0 {
            guarded.try_get_initial_img()
        } else {
            guarded.try_get_step_img(self.step_num - 1)
        };

        let cropped_copy: Img;
        if let Some(crop_area) = self.crop_area {
            cropped_copy = img_to_process.get_cropped_copy(crop_area);
            img_to_process = &cropped_copy;
        }

        let step = &guarded.proc_steps[self.step_num];
        let img_result = match step.filter.filter(&img_to_process, &mut prog_prov) {
            Ok(img) => {
                assert!(prog_prov.all_actions_completed());
                Some(img)
            },
            Err(_halted) => None
        };

        guarded.proc_steps[self.step_num].img = img_result;

        Ok(())
    }
}

impl Task for ProcTask {
    fn is_completed(&self) -> bool {
        self.result.is_some()
    }

    fn complete(&mut self, guarded: &mut Guarded) {
        self.result = Some(self.process(guarded));
        guarded.tx_notify.send(TaskMsg::Finished).unwrap();

        assert!(guarded.has_initial_img());
    }

    fn take_result(&mut self) -> Result<(), MyError> {
        self.result.take().unwrap()
    }
}


#[derive(Debug)]
pub struct ExportTask {
    dir_path: String,
    result: Option<Result<(), MyError>>
}

impl ExportTask {
    pub fn new(dir_path: String) -> TaskBase {
        let task = ExportTask {
            dir_path,
            result: None
        };
        Box::new(task) as TaskBase
    }

    fn export(&self, guarded: &mut Guarded) -> Result<(), MyError> {
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

impl Task for ExportTask {
    fn is_completed(&self) -> bool {
        self.result.is_some()
    }

    fn complete(&mut self, guarded: &mut Guarded) {
        self.result = Some(self.export(guarded));
        guarded.tx_notify.send( TaskMsg::Finished ).unwrap();
    }

    fn take_result(&mut self) -> Result<(), MyError> {
        self.result.take().unwrap()
    }
}
