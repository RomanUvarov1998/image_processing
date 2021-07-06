use std::{sync::mpsc::Receiver, time};
use fltk::app::{Sender};
use crate::message::{Msg, Proc};


pub struct Halted;

pub struct HaltMessage;

pub struct ProgressProvider<'own> {
    tx_progress: &'own Sender<Msg>,
    step_num: usize,
    total_steps: usize,
    rx_halt: &'own Receiver<HaltMessage>,
    actions_total: usize,
    actions_completed: usize,
    prev_time: time::Instant,
}

impl<'own> ProgressProvider<'own> {
    pub fn new(tx_progress: &'own Sender<Msg>, rx_halt: &'own Receiver<HaltMessage>, step_num: usize, total_steps: usize) -> Self {
        ProgressProvider { 
            tx_progress, 
            step_num, 
            total_steps,
            rx_halt,
            actions_total: 0,
            actions_completed: 0,
            prev_time: time::Instant::now(),
        }
    }

    pub fn reset_and_set_total_actions_count(&mut self, actions_count: usize) {
        self.actions_completed = 0;
        self.actions_total = actions_count;
    }

    const MS_DELAY: u128 = 100;

    pub fn complete_action(&mut self) -> Result<(), Halted> { 
        self.actions_completed += 1;
        
        if self.prev_time.elapsed().as_millis() > Self::MS_DELAY {
            if let Ok(_) = self.rx_halt.try_recv() {
                return Err(Halted);
            }
            
            self.prev_time = time::Instant::now();
            self.send_progress_msg();
        }

        return Ok(());
    }

    pub fn all_actions_completed(&self) -> bool {
        self.actions_total == self.actions_completed
    }

    fn send_progress_msg(&mut self) {
        let step_percents = self.actions_completed * 100 / self.actions_total;
        let total_percents = (self.step_num * 100 + step_percents) / self.total_steps;
        self.tx_progress.send( Msg::Proc( Proc::StepProgress { step_num: self.step_num, step_percents, total_percents } ) );
    }
}