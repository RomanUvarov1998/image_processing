use crate::{filter::{filter_trait::{Filter}}};

mod progress_provider;
mod background_worker;

pub use background_worker::BackgroundWorker;
pub use background_worker::StartProcResult;
pub use background_worker::StartResultsSavingResult;
pub use progress_provider::ProgressProvider;
pub use progress_provider::Halted;

pub type FilterBase = Box<dyn Filter>;
