use std::{ops::{Deref, DerefMut}, sync::{Arc, Mutex}};

use crate::my_err::MyError;

pub struct Halted;


pub fn task_info_channel() -> (ExecutorHandle, DelegatorHandle) {
	let inner = Arc::new(Mutex::new(TaskState::Empty));
	(ExecutorHandle::new(&inner), DelegatorHandle::new(&inner))
}


pub struct ExecutorHandle {
	actions_completed: usize,
	actions_total: usize,
	inner: Arc<Mutex<TaskState>>
}

impl ExecutorHandle {
	fn new(inner: &Arc<Mutex<TaskState>>) -> Self {
		ExecutorHandle {
			actions_completed: 0,
			actions_total: 0,
			inner: Arc::clone(inner),
		}
	}

	pub fn reset(&mut self, actions_total: usize) {
		let mut guard = self.inner.lock().unwrap();
		let state: &mut TaskState = guard.deref_mut();

		print!("reset ");
		self.actions_total = actions_total;
		self.actions_completed = 0;

		state.assert_can_start_new_task();
		*state = TaskState::InProgress { percents: 0 };

		drop(guard);
	}

	pub fn get_task_state(&self) -> TaskState {
		self.inner.lock().unwrap().deref().clone()
	}

	pub fn complete_action(&mut self) -> Result<(), Halted> {		
		let mut guard = self.inner.lock().unwrap();
		let state: &mut TaskState = guard.deref_mut();

		let result: Result<(), Halted> = 
			match state {
				TaskState::Empty => panic!("Task is empty!"),
				TaskState::InProgress { .. } => {
					assert!(self.actions_completed < self.actions_total);

					self.actions_completed += 1;

					let percents: usize = self.actions_completed * 100 / self.actions_total;
					*state = TaskState::InProgress { percents };
					
					Ok(())
				},
				TaskState::Finished { result } => {
					match result {
						TaskResult::Ok | TaskResult::Err(_) => unreachable!(),
						TaskResult::Halted => {
							println!("HALT DETECTED");
							Err(Halted)
						},
					}
				},
			};

		drop(guard);

		result
	}

	pub fn finish_task(&self, result: Result<(), MyError>) {	
		let mut guard = self.inner.lock().unwrap();
		let state: &mut TaskState = guard.deref_mut();
		match state {
			TaskState::InProgress { percents } => assert_eq!(*percents, 100),
			TaskState::Empty | TaskState::Finished { .. } => unreachable!(),
		}
		*state = TaskState::finish_with_result(result);
		drop(guard);
	}

	pub fn assert_all_actions_completed(&self) {
		if self.actions_completed != self.actions_total {
            panic!("not all acions completed: {} of {}", self.actions_completed, self.actions_total);
        }
	}
}


#[derive(Debug)]
pub struct DelegatorHandle {
	inner: Arc<Mutex<TaskState>>
}

impl DelegatorHandle {
	fn new(inner: &Arc<Mutex<TaskState>>) -> Self {
		DelegatorHandle {
			inner: Arc::clone(inner)
		}
	}

	pub fn halt_task(&self) {
		let mut guard = self.inner.lock().unwrap();
		let state: &mut TaskState = guard.deref_mut();
		*state = TaskState::Finished { result: TaskResult::Halted };
		drop(guard);
	}

	pub fn clear_task(&self) {
		let mut guard = self.inner.lock().unwrap();
		let state: &mut TaskState = guard.deref_mut();
		*state = TaskState::Empty;
		drop(guard);
	}

	pub fn get_task_state(&self) -> TaskState {
		self.inner.lock().unwrap().deref().clone()
	}
}


#[derive(Clone, Debug)]
pub enum TaskState {
	Empty,
	InProgress { percents: usize },
	Finished { result: TaskResult },
}

impl TaskState {
	fn assert_can_start_new_task(&self) {
		match self {
			TaskState::Empty | TaskState::Finished { .. } => {},
			TaskState::InProgress { .. } => panic!("Previous task is not completed!"),
		}
	}

	fn finish_with_result(result: Result<(), MyError>) -> Self {
		let result: TaskResult = match result {
			Ok(()) => TaskResult::Ok,
			Err(err) => TaskResult::Err(err),
		};
		TaskState::Finished { result }
	}
}

#[derive(Clone, Debug)]
pub enum TaskResult {
	Ok,
	Err(MyError),
	Halted
}