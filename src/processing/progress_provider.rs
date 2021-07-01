use std::{sync::mpsc::Receiver, time};
use fltk::app::{Sender};
use crate::message::{Message, Processing};


pub struct HaltError;

pub struct HaltMessage;

pub struct ProgressProvider<'own> {
    sender: &'own Sender<Message>,
    step_num: usize,
    halt_msg_receiver: &'own Receiver<HaltMessage>,
    actions_total: usize,
    actions_completed: usize,
    prev_time: time::Instant,
}

impl<'own> ProgressProvider<'own> {
    pub fn new(sender: &'own Sender<Message>, halt_msg_receiver: &'own Receiver<HaltMessage>, step_num: usize) -> Self {
        ProgressProvider { 
            sender, 
            step_num, 
            halt_msg_receiver,
            actions_total: 0,
            actions_completed: 0,
            prev_time: time::Instant::now(),
        }
    }

    pub fn reset(&mut self, actions_count: usize) {
        self.actions_completed = 0;
        self.actions_total = actions_count;
    }

    const MS_DELAY: u128 = 100;

    pub fn complete_action(&mut self) -> Result<(), HaltError> { 
        if let Ok(_) = self.halt_msg_receiver.try_recv() {
            return Err(HaltError);
        }

        self.actions_completed += 1;

        if self.prev_time.elapsed().as_millis() > Self::MS_DELAY {
            self.prev_time = time::Instant::now();
            let cur_percents = self.actions_completed * 100 / self.actions_total;
            let step_num = self.step_num;
            self.sender.send(Message::Processing(Processing::StepProgress{ step_num, cur_percents }));
        }

        return Ok(());
    }

    pub fn all_actions_completed(&self) -> bool {
        self.actions_total == self.actions_completed
    }
}