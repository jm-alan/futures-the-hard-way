use core::future::Future;
use std::{
  collections::VecDeque,
  sync::{Arc, Mutex},
  time::Instant,
};

use crate::task::{InnerTask, Task, TaskError, Taskable, task_pair};

pub struct Executor {
  tasks: Mutex<VecDeque<InnerTask<Box<dyn Taskable + 'static>>>>,
  last_checked_timers: Mutex<Instant>,
}

impl Executor {
  fn new() -> Arc<Self> {
    Arc::new(Self {
      tasks: Mutex::new(VecDeque::new()),
      last_checked_timers: Mutex::new(Instant::now()),
    })
  }

  #[inline(always)]
  pub fn main<Main, FWrap, Result>(_: Main) -> Result
  where
    Main: FnOnce(Arc<Executor>) -> FWrap,
    FWrap: IntoFuture<Output = Result>,
  {
    todo!("");
  }

  #[inline(always)]
  pub fn spawn<T>(
    self: &mut Arc<Self>,
    f: impl Future<Output = T> + Send + 'static,
  ) -> Result<Task<T>, TaskError>
  where
    T: Taskable + 'static,
  {
    let Ok(mut ttex) = self.tasks.lock() else {
      return Err(TaskError);
    };

    let (inner, outer) = task_pair(f);

    ttex.push_back(unsafe {
      (&inner as *const _ as *const InnerTask<Box<dyn Taskable + 'static>>).read()
    });

    Ok(outer)
  }
}
