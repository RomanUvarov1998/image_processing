use std::{sync::{Arc, Condvar, Mutex}, thread::{self, JoinHandle}};

use crate::{img::Img, message::{Message, Processing}};
use super::{FilterBase, progress_provider::{HaltMessage, ProgressProvider}};


pub struct BackgroundWorker {
    inner: Arc<Inner>,
    _processing_thread_handle: JoinHandle<()>
}

impl BackgroundWorker {
    pub fn new(progress_tx: fltk::app::Sender<Message>, halt_msg_rx: std::sync::mpsc::Receiver<HaltMessage>) -> Self {
        let inner = Arc::new(Inner::new());

        let inner_arc = Arc::clone(&inner);
        let _processing_thread_handle: JoinHandle<()> = thread::Builder::new()
            .name("Processing".to_string())
            .spawn(move || 
        {
            loop {
                let mut guard = inner_arc.guarded.lock().expect("Couldn't lock");

                guard = inner_arc.cv.wait_while(guard, |g| g.has_task() == false).expect("Couldn't wait");

                let task = guard.take_task();

                let mut prog_prov = ProgressProvider::new(&progress_tx, &halt_msg_rx);
                prog_prov.set_step_num(task.step_num);
                
                let img_result = match task.filter_copy.filter(&task.img, &mut prog_prov) {
                    Ok(img) => {
                        assert!(prog_prov.completed());
                        Some(img)
                    },
                    Err(_) => None
                };

                let task_result = TaskResult {
                    step_num: task.step_num,
                    img: img_result,
                    do_until_end: task.do_until_end,
                };

                let step_num = task.step_num;

                guard.put_result(task_result);

                drop(guard);

                progress_tx.send(Message::Processing(Processing::StepIsCompleted { step_num }));
            }
        })
            .expect("Couldn't create a processing thread");

        BackgroundWorker { inner, _processing_thread_handle }
    }

    pub fn put_task(&mut self, step_num: usize, filter_copy: FilterBase, img: Img, do_until_end: bool) {
        let task = Task { step_num, filter_copy, img, do_until_end };

        let mut guard = self.inner.guarded.lock().expect("Couldn't lock");
        guard.put_task(task);
        drop(guard);

        self.inner.cv.notify_one();
    }

    pub fn take_result(&mut self) -> TaskResult {
        let mut guard = self.inner.guarded.lock().expect("Couldn't lock");
        guard.take_task_result()
    }
}


struct Inner {
    cv: Condvar,
    guarded: Mutex<Guarded>
}

impl Inner {
    fn new() -> Self {
        Inner {
            cv: Condvar::new(),
            guarded: Mutex::new(Guarded::Empty)
        }
    }
}

enum Guarded {
    Empty,
    HasTask(Option<Task>),
    HasResult(Option<TaskResult>)
}

struct Task { step_num: usize, filter_copy: FilterBase, img: Img, do_until_end: bool }
pub struct TaskResult { 
    pub step_num: usize, 
    pub img: Option<Img>, 
    pub do_until_end: bool 
}

impl Guarded {
    fn has_task(&mut self) -> bool {
        match self {
            Guarded::Empty | Guarded::HasResult (_) => false,
            Guarded::HasTask (_) => true,
        }
    }

    fn put_task(&mut self, task: Task) {
        *self = Guarded::HasTask(Some(task));
    }

    fn take_task(&mut self) -> Task {
        match self {
            Guarded::Empty | Guarded::HasResult (_) => unreachable!(),
            Guarded::HasTask (task) => task.take().expect("didn't found task"),
        }
    }

    fn put_result(&mut self, task_result: TaskResult) {
        *self = Guarded::HasResult(Some(task_result));
    }

    fn take_task_result(&mut self) -> TaskResult {
        match self {
            Guarded::Empty | Guarded::HasTask (_) => unreachable!(),
            Guarded::HasResult (result) => result.take().expect("didn't found task result"),
        }
    }
}