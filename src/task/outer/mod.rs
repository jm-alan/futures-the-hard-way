use std::{
  pin::Pin,
  sync::Arc,
  task::{Context, Poll},
};

use super::{TaskError, TaskState};

pub trait Taskable: Send + 'static {}
impl<T> Taskable for T where T: Send + 'static {}

#[derive(Clone)]
pub struct Task<T: Taskable> {
  state: Arc<TaskState<Box<T>>>,
}

impl<T: Taskable> Task<T> {
  #[inline(always)]
  pub(crate) fn new(state: Arc<TaskState<Box<T>>>) -> Self {
    Self { state }
  }

  #[inline(always)]
  pub(crate) fn resolve(&mut self, val: Box<T>) -> Result<(), TaskError> {
    let (Ok(mut rtex), Ok(mut wtex)) = (self.state.resolved.lock(), self.state.waker.lock()) else {
      return Err(TaskError);
    };

    rtex.replace(val);

    if let Some(waker) = wtex.take() {
      waker.wake();
    }

    Ok(())
  }
}

impl<T: Taskable> Future for Task<T> {
  type Output = Result<T, TaskError>;
  fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    let Ok(mut rtex) = self.state.resolved.lock() else {
      return Poll::Ready(Err(TaskError));
    };

    let Some(resolved) = rtex.take() else {
      let Ok(mut wtex) = self.state.waker.lock() else {
        return Poll::Ready(Err(TaskError));
      };
      if let Some(old) = wtex.replace(cx.waker().clone()) {
        drop(old);
      };
      return Poll::Pending;
    };

    Poll::Ready(Ok(*resolved))
  }
}
