mod progress_provider;
mod background_worker;
mod guarded;
mod task_info_channel;

#[cfg(test)]
mod tests;

pub use task_info_channel::{task_info_channel, ExecutorHandle, DelegatorHandle};
pub use background_worker::BackgroundWorker;
pub use progress_provider::ProgressProvider;
pub use progress_provider::Halted;
pub use progress_provider::HaltMessage;
pub use guarded::StartProcResult;
pub use guarded::StartResultsSavingResult;
pub use guarded::TaskSetup;
pub use guarded::PROJECT_EXT;

#[derive(Debug, Copy, Clone)]
pub enum TaskMsg {
    Progress { percents: usize },
    Finished,
}
