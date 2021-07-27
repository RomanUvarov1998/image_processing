mod background_worker;
mod guarded;
mod task_info_channel;

#[cfg(test)]
mod tests;

pub use background_worker::BackgroundWorker;
pub use guarded::StartProcResult;
pub use guarded::StartResultsSavingResult;
pub use guarded::TaskSetup;
pub use guarded::PROJECT_EXT;
pub use task_info_channel::{
    create_task_info_channel, DelegatorHandle, ExecutorHandle, TaskState, TaskStop,
};
