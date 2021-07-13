mod progress_provider;
mod background_worker;
mod guarded;

#[cfg(test)]
mod tests;

pub use background_worker::BackgroundWorker;
pub use progress_provider::ProgressProvider;
pub use progress_provider::Halted;
pub use guarded::StartProcResult;
pub use guarded::StartResultsSavingResult;
pub use guarded::tasks::*;

#[derive(Debug, Copy, Clone)]
pub enum TaskMsg {
    Progress { percents: usize },
    Finished,
}
