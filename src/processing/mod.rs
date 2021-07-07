use crate::{filter::{filter_trait::{Filter}}};

mod progress_provider;
mod background_worker;
mod guarded;
mod task;

#[cfg(test)]
mod tests;

pub use background_worker::BackgroundWorker;
pub use progress_provider::ProgressProvider;
pub use progress_provider::Halted;
pub use guarded::StartProcResult;
pub use guarded::StartResultsSavingResult;
pub use task::*;

pub type FilterBase = Box<dyn Filter>;

#[derive(Debug, Copy, Clone)]
pub enum TaskMsg {
    Progress { percents: usize },
    Finished,
}
