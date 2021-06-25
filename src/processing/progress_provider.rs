use std::{sync::mpsc::Receiver, time};
use fltk::app::{Sender};
use crate::message::{Message, Processing};


pub struct HaltError;

pub struct HaltMessage;

pub struct ProgressProvider {
    sender: Sender<Message>,
    pr_data: Option<ProgressData>,
    step_num: Option<usize>,
    halt_msg_receiver: Option<Receiver<HaltMessage>>
}

impl ProgressProvider {
    pub fn new(sender: Sender<Message>, receiver: Receiver<HaltMessage>) -> Self {
        ProgressProvider { sender, pr_data: None, step_num: None, halt_msg_receiver: Some(receiver) }
    }

    pub fn set_step_num(&mut self, step_num: usize) {
        self.step_num = Some(step_num);
    }

    pub fn reset(&mut self, actions_count: usize) {
        // to not to panic if previous progress was not completed, otherwise destructor
        // will panic if not all steps are completed
        self.drop_progress_data();
        
        self.pr_data = Some(ProgressData::new(actions_count));
    }

    pub fn drop_progress_data(&mut self) {
        if let Some(ref mut pd) = self.pr_data {
            pd.finish();
        }
    }

    const MS_DELAY: u128 = 100;

    pub fn complete_action(&mut self) -> Result<(), HaltError> { 
        if let Ok(_) = self.halt_msg_receiver.as_ref().unwrap().try_recv() {
            return Err(HaltError);
        }

        match self.pr_data {
            Some(ref mut data) => {
                data.completed_actions_count += 1;

                if data.prev_time.elapsed().as_millis() > Self::MS_DELAY {
                    data.prev_time = time::Instant::now();
                
                    let cur_percents = data.completed_actions_count * 100 / data.all_actions_count;
                
                    let step_num = self.step_num.unwrap();
                
                    self.sender.send(Message::Processing(Processing::StepProgress{ step_num, cur_percents }));
                }

                return Ok(());
            },
            None => panic!("No process data!"),
        }      
    }

    pub fn take_receiver(&mut self) -> Receiver<HaltMessage> {
        self.halt_msg_receiver.take().unwrap()
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