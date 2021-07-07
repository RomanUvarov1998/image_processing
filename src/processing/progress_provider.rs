use std::{sync::mpsc::Receiver, time};
use crate::message::TaskMsg;


pub struct Halted;

pub struct HaltMessage;

pub struct ProgressProvider<'own> {
    tx_progress: &'own std::sync::mpsc::Sender<TaskMsg>,
    rx_halt: &'own Receiver<HaltMessage>,
    actions_total: usize,
    actions_completed: usize,
    prev_time: time::Instant,
}

impl<'own> ProgressProvider<'own> {
    pub fn new(tx_progress: &'own std::sync::mpsc::Sender<TaskMsg>, rx_halt: &'own Receiver<HaltMessage>) -> Self {
        ProgressProvider { 
            tx_progress, 
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
        let percents = self.actions_completed * 100 / self.actions_total;
        self.tx_progress.send( TaskMsg::Progress { percents } ).unwrap();
    }
}