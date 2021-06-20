use std::time;
use fltk::app::Sender;
use crate::message::{Message, Processing};

pub struct ProgressProvider {
    sender: Sender<Message>,
    pr_data: Option<ProgressData>,
    step_num: Option<usize>
}

impl ProgressProvider {
    pub fn new(sender: Sender<Message>) -> Self {
        ProgressProvider { sender, pr_data: None, step_num: None }
    }

    pub fn set_step_num(&mut self, step_num: usize) {
        self.step_num = Some(step_num);
    }

    pub fn reset(&mut self, actions_count: usize) {
        // to not to panic if previous progress was not completed, otherwise destructor
        // will panic if not all steps are completed
        if let Some(ref mut pd) = self.pr_data {
            pd.finish();
        }

        self.pr_data = Some(ProgressData::new(actions_count));
    }

    const MS_DELAY: u128 = 100;

    pub fn complete_action(&mut self) {          
        match self.pr_data {
            Some(ref mut data) => {
                data.completed_actions_count += 1;

                if data.prev_time.elapsed().as_millis() > Self::MS_DELAY {
                    data.prev_time = time::Instant::now();
                
                    let cur_percents = data.completed_actions_count * 100 / data.all_actions_count;
                
                    let step_num = self.step_num.unwrap();
                
                    self.sender.send(Message::Processing(Processing::StepProgress{ step_num, cur_percents }));
                }
            },
            None => panic!("No process data!"),
        }      

        self.print_completed_actions_count();
    }

    #[allow(unused)]
    pub fn print_completed_actions_count(&self) {
        if let Some(ref pd) = self.pr_data {
            if (pd.completed_actions_count > pd.all_actions_count) {
                println!("completed {} actions of {}", pd.completed_actions_count, pd.all_actions_count);
                panic!("data.completed_actions_count > data.all_actions_count");
            }
        }
    }
}


struct ProgressData {
    all_actions_count: usize,
    completed_actions_count: usize,
    prev_time: time::Instant,
}

impl ProgressData {
    fn new(all_actions_count: usize) -> Self {
        ProgressData {
            all_actions_count, completed_actions_count: 0,
            prev_time: time::Instant::now(),
        }
    }

    fn finish(&mut self) {
        self.completed_actions_count = self.all_actions_count;
    }
}

impl Drop for ProgressData {
    fn drop(&mut self) {
        assert_eq!(
            self.all_actions_count, self.completed_actions_count, 
            "not all actions are completed {} of {}!", 
            self.completed_actions_count, self.all_actions_count);
    }
}