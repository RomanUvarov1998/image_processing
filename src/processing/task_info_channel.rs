use std::{
    ops::{Deref, DerefMut},
    sync::{Arc, Mutex},
};

use fltk::prelude::FltkError;

use crate::my_err::MyError;

pub fn create_task_info_channel() -> (ExecutorHandle, DelegatorHandle) {
    let inner = Arc::new(Mutex::new(TaskState::Empty));
    (ExecutorHandle::new(&inner), DelegatorHandle::new(&inner))
}

pub struct ExecutorHandle {
    actions_completed: usize,
    actions_total: usize,
    inner: Arc<Mutex<TaskState>>,
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

        match state {
            TaskState::Empty => *state = TaskState::InProgress { percents: 0 },
            TaskState::InProgress { .. } => panic!("Previous task wasn't completed"),
            TaskState::Finished { .. } => panic!("Previous task's result wasn't taken"),
        }

        drop(guard);
    }

    pub fn get_task_state(&self) -> TaskState {
        self.inner.lock().unwrap().deref().clone()
    }

    pub fn complete_action(&mut self) -> Result<(), TaskStop> {
        let mut guard = self.inner.lock().unwrap();
        let state: &mut TaskState = guard.deref_mut();

        let result: Result<(), TaskStop> = match state {
            TaskState::Empty => panic!("Task is empty!"),
            TaskState::InProgress { .. } => {
                assert!(self.actions_completed < self.actions_total);

                self.actions_completed += 1;

                let percents: usize = self.actions_completed * 100 / self.actions_total;
                *state = TaskState::InProgress { percents };

                Ok(())
            }
            TaskState::Finished { result } => result.clone(),
        };

        drop(guard);

        result
    }

    pub fn finish_task(&self, result: Result<(), TaskStop>) {
        let mut guard = self.inner.lock().unwrap();
        let state: &mut TaskState = guard.deref_mut();
        match state {
            TaskState::InProgress { percents } => {
                if let Ok(_) = result {
                    self.assert_all_actions_completed();
                    assert_eq!(*percents, 100);
                }
            }
            TaskState::Empty | TaskState::Finished { .. } => unreachable!(),
        }
        *state = TaskState::Finished { result };
        drop(guard);
    }

    pub fn assert_all_actions_completed(&self) {
        if self.actions_completed != self.actions_total {
            panic!(
                "not all acions completed: {} of {}",
                self.actions_completed, self.actions_total
            );
        }
    }
}

#[derive(Debug)]
pub struct DelegatorHandle {
    inner: Arc<Mutex<TaskState>>,
}

impl DelegatorHandle {
    fn new(inner: &Arc<Mutex<TaskState>>) -> Self {
        DelegatorHandle {
            inner: Arc::clone(inner),
        }
    }

    pub fn halt_task(&self) {
        let mut guard = self.inner.lock().unwrap();
        let state: &mut TaskState = guard.deref_mut();
        *state = TaskState::Finished {
            result: Err(TaskStop::Halted),
        };
        drop(guard);
    }

    pub fn get_task_state(&self) -> TaskState {
        let guard = self.inner.lock().unwrap();
        guard.deref().clone()
    }

    pub fn clear_task(&self) {
        let mut guard = self.inner.lock().unwrap();
        let state: &mut TaskState = guard.deref_mut();
        *state = TaskState::Empty;
        drop(guard);
    }

    pub fn get_task_result(&self) -> Result<(), TaskStop> {
        let mut guard = self.inner.lock().unwrap();
        let state: &mut TaskState = guard.deref_mut();
        let result = state.take_result();
        drop(guard);
        result
    }
}

#[derive(Clone, Debug)]
pub enum TaskState {
    Empty,
    InProgress { percents: usize },
    Finished { result: Result<(), TaskStop> },
}

impl TaskState {
    fn take_result(&mut self) -> Result<(), TaskStop> {
        match self {
            TaskState::Empty | TaskState::InProgress { .. } => panic!("No task result yet!"),
            TaskState::Finished { result } => {
                let res = result.clone();
                *self = TaskState::Empty;
                res
            }
        }
    }
}

#[derive(Clone, Debug)]
pub enum TaskStop {
    Err(MyError),
    Halted,
}

impl From<std::io::Error> for TaskStop {
    fn from(err: std::io::Error) -> Self {
        TaskStop::Err(err.into())
    }
}

impl From<MyError> for TaskStop {
    fn from(err: MyError) -> Self {
        TaskStop::Err(err)
    }
}

impl From<FltkError> for TaskStop {
    fn from(err: FltkError) -> Self {
        TaskStop::Err(err.into())
    }
}
