use std::{sync::mpsc::Receiver, time};
use fltk::app::{Sender};
use crate::message::{Message, Processing};


pub struct HaltError;

pub struct HaltMessage;

pub struct ProgressProvider<'own> {
    sender: &'own Sender<Message>,
    pr_data: ProgressData,
    step_num: usize,
    halt_msg_receiver: &'own Receiver<HaltMessage>
}

impl<'own> ProgressProvider<'own> {
    pub fn new(sender: &'own Sender<Message>, halt_msg_receiver: &'own Receiver<HaltMessage>, step_num: usize) -> Self {
        ProgressProvider { 
            sender, 
            pr_data: ProgressData::new(), 
            step_num, 
            halt_msg_receiver 
        }
    }

    pub fn reset(&mut self, actions_count: usize) {
        self.pr_data.all_actions_count = actions_count;
    }

    const MS_DELAY: u128 = 100;

    pub fn complete_action(&mut self) -> Result<(), HaltError> { 
        if let Ok(_) = self.halt_msg_receiver.try_recv() {
            return Err(HaltError);
        }

        self.pr_data.completed_actions_count += 1;

        if self.pr_data.prev_time.elapsed().as_millis() > Self::MS_DELAY {
            self.pr_data.prev_time = time::Instant::now();
        
            let cur_percents = self.pr_data.completed_actions_count * 100 / self.pr_data.all_actions_count;
        
            let step_num = self.step_num;
        
            self.sender.send(Message::Processing(Processing::StepProgress{ step_num, cur_percents }));
        }

        return Ok(());
    }

    pub fn completed(&self) -> bool {
        self.pr_data.all_actions_count == self.pr_data.completed_actions_count
    }

    #[allow(unused)]
    pub fn print_completed_actions_count(&self) {
        if (self.pr_data.completed_actions_count > self.pr_data.all_actions_count) {
            println!("completed {} actions of {}", self.pr_data.completed_actions_count, self.pr_data.all_actions_count);
            panic!("data.completed_actions_count > data.all_actions_count");
        }
    }
}


struct ProgressData {
    all_actions_count: usize,
    completed_actions_count: usize,
    prev_time: time::Instant,
}

impl ProgressData {
    fn new() -> Self {
        ProgressData {
            all_actions_count: 0, 
            completed_actions_count: 0,
            prev_time: time::Instant::now(),
        }
    }
}