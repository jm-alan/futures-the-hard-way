mod error;
mod inner;
mod outer;

use std::{
  sync::{Arc, Mutex},
  task::Waker,
};

pub use error::TaskError;
pub(crate) use inner::InnerTask;
pub use outer::Task;
pub(crate) use outer::Taskable;

pub(crate) struct TaskState<T: Taskable> {
  resolved: Mutex<Option<T>>,
  waker: Mutex<Option<Waker>>,
}

impl<T: Taskable> TaskState<T> {
  pub fn new() -> Self {
    Self {
      resolved: Mutex::new(None),
      waker: Mutex::new(None),
    }
  }
}

pub(crate) fn task_pair<T: Taskable + 'static>(
  f: impl Future<Output = T> + Send + 'static,
) -> (InnerTask<Box<T>>, Task<T>) {
  let inner_state = Arc::new(TaskState::new());
  let outer_state = inner_state.clone();
  (
    InnerTask {
      fut: Mutex::new(Box::pin(async { Box::new(f.await) })),
      state: inner_state,
    },
    Task::new(outer_state),
  )
}
