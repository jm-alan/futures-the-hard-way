mod error;
mod inner;
mod outer;

use std::task::Waker;

pub use error::TaskError;
pub(crate) use inner::InnerTask;
pub use outer::Task;
pub(crate) use outer::Taskable;

pub(crate) struct TaskState<T: Taskable> {
  resolved: Option<T>,
  waker: Option<Waker>,
}

impl<T: Taskable> TaskState<T> {
  #[inline(always)]
  pub fn new() -> Self {
    Self {
      resolved: None,
      waker: None,
    }
  }

  #[inline(always)]
  pub(crate) fn set(&mut self, val: T) {
    if self.resolved.is_some() {
      return;
    }

    self.resolved.replace(val);

    if let Some(waker) = self.waker.take() {
      waker.wake();
    }
  }
}
