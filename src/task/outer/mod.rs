use std::{
  fmt::Debug,
  pin::Pin,
  sync::{Arc, Mutex},
  task::{Context, Poll},
};

use super::{TaskError, TaskState};

pub trait Taskable: Send + 'static {}
impl<T> Taskable for T where T: Send + 'static {}

#[derive(Clone)]
pub struct Task<T: Taskable> {
  id: usize,
  state: Arc<Mutex<TaskState<T>>>,
}

impl<T: Taskable> Debug for Task<T> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(&format!("Task<{}>", self.id))
  }
}

impl<T: Taskable> Task<T> {
  #[inline(always)]
  pub(crate) fn new(id: usize, state: Arc<Mutex<TaskState<T>>>) -> Self {
    Self { id, state }
  }
}

impl<T: Taskable> Future for Task<T> {
  type Output = Result<T, TaskError>;

  fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    let Ok(mut stex) = self.state.lock() else {
      return Poll::Ready(Err(TaskError));
    };

    if let Some(resolved) = stex.resolved.take() {
      return Poll::Ready(Ok(resolved));
    }

    if let Some(old) = stex.waker.replace(cx.waker().clone()) {
      drop(old);
    };

    Poll::Pending
  }
}
