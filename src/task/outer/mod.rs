use std::{
  pin::Pin,
  sync::{Arc, Mutex},
  task::{Context, Poll},
};

use super::{TaskError, TaskState};

pub trait Taskable: Send + 'static {}
impl<T> Taskable for T where T: Send + 'static {}

pub struct Task<T: Taskable> {
  state: Arc<Mutex<TaskState<T>>>,
}

impl<T: Taskable> Clone for Task<T> {
  fn clone(&self) -> Self {
    Self {
      state: self.state.clone(),
    }
  }
}

impl<T: Taskable> Task<T> {
  #[inline(always)]
  pub(crate) fn new(state: Arc<Mutex<TaskState<T>>>) -> Self {
    Self { state }
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
