use std::time;

pub struct ProgressProvider<Cbk: Fn(usize)> {
    all_actions_count: usize,
    completed_actions_count: usize,
    prev_time: time::Instant,
    progress_cbk: Cbk
}

#[allow(unused)]
impl<Cbk: Fn(usize)> ProgressProvider<Cbk> {
    pub fn new(progress_cbk: Cbk, actions_count: usize) -> Self {
        ProgressProvider::<Cbk> {
            all_actions_count: actions_count,
            completed_actions_count: 0,
            prev_time: time::Instant::now(),
            progress_cbk
        }
    }

    pub fn start(&mut self) {
        self.prev_time = time::Instant::now();
    }

    const MS_DELAY: u128 = 100;

    pub fn complete_action(&mut self) {
        self.completed_actions_count += 1;
        if self.prev_time.elapsed().as_millis() > Self::MS_DELAY {
            self.prev_time = time::Instant::now();
            (self.progress_cbk)(self.completed_actions_count * 100 / self.all_actions_count);
        }
    }

    pub fn print_completed_actions_count(&self) {
        println!("completed {} actions of {}", self.completed_actions_count, self.all_actions_count);
    }
}

impl<Cbk: Fn(usize)> Drop for ProgressProvider<Cbk> {
    fn drop(&mut self) {
        assert_eq!(
            self.all_actions_count, self.completed_actions_count, 
            "not all actions are completed {} of {}!", 
            self.completed_actions_count, self.all_actions_count);
    }
}