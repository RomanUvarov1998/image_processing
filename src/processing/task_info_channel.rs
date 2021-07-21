use std::{cell::Cell, sync::{Arc, atomic::{AtomicBool, AtomicUsize, Ordering}}};

use super::Halted;


pub fn task_info_channel() -> (ExecutorHandle, DelegatorHandle) {
	let inner = Arc::new(TaskInfo::new());
	(ExecutorHandle::new(&inner), DelegatorHandle::new(&inner))
}


pub struct ExecutorHandle {
	inner: Arc<TaskInfo>
}

impl ExecutorHandle {
	fn new(inner: &Arc<TaskInfo>) -> Self {
		ExecutorHandle {
			inner: Arc::clone(inner)
		}
	}

	pub fn reset(&self, actions_total: usize) {
		if !self.task_is_halted() {
			assert!(self.completed() == self.total());
		}

		print!("reset ");
		self.inner.actions_total.set(actions_total);
		self.inner.actions_completed.set(0);
		self.inner.percents.store(0, Ordering::Relaxed);
		self.inner.is_halted.store(false, Ordering::Relaxed);
	}

	pub fn task_is_halted(&self) -> bool {
		self.inner.is_halted.load(Ordering::Relaxed)
	}

	pub fn complete_action(&self) -> Result<(), Halted> {
		if self.task_is_halted() {
			println!("HALT DETECTED");
			Err(Halted)
		} else {
			let (mut completed, total) = (self.completed(), self.total());
			assert!(completed < total);

			completed += 1;
			self.inner.actions_completed.set(completed);

			let percents = completed * 100 / total;
			self.inner.percents.store(percents, Ordering::Relaxed);
			Ok(())
		}
	}

	pub fn assert_all_actions_completed(&self) {
		let (completed, total) = (self.completed(), self.total());
		if completed != total {
            panic!("not all acions completed: {} of {}", completed, total);
        }
	}

	fn total(&self) -> usize {
		self.inner.actions_total.get()
	}

	fn completed(&self) -> usize {
		self.inner.actions_completed.get()
	}
}


#[derive(Debug)]
pub struct DelegatorHandle {
	inner: Arc<TaskInfo>
}

impl DelegatorHandle {
	fn new(inner: &Arc<TaskInfo>) -> Self {
		DelegatorHandle {
			inner: Arc::clone(inner)
		}
	}

	pub fn halt_task(&self) {
		self.inner.is_halted.store(true, Ordering::Relaxed);
	}

	pub fn task_is_halted(&self) -> bool {
		self.inner.is_halted.load(Ordering::Relaxed)
	}

	pub fn get_completed_percents(&self) -> usize {
		self.inner.percents.load(Ordering::Relaxed)
	}
}


#[derive(Debug)]
struct TaskInfo {
	actions_completed: Cell<usize>,
	actions_total: Cell<usize>,
	percents: AtomicUsize,
	is_halted: AtomicBool
}

impl TaskInfo {
	fn new() -> Self {
		TaskInfo {
			actions_completed: Cell::new(0),
			actions_total: Cell::new(0),
			percents: AtomicUsize::new(0),
			is_halted: AtomicBool::new(false)
		}
	}
}

unsafe impl Sync for TaskInfo {}