use std::{
  error::Error,
  fmt::Display,
  pin::Pin,
  sync::{Arc, Mutex},
  task::{Context, Poll, Wake},
};

type BoxFuture<T> = Pin<Box<dyn Future<Output = Box<T>> + Send + 'static>>;

pub(crate) trait Taskable {}

impl<T> Taskable for T where T: Sized {}

#[derive(Debug)]
pub struct TaskError;

impl Display for TaskError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str("<Task internal mutex poisoned>")
  }
}

impl Error for TaskError {}

pub struct Task<T: ?Sized + Send + 'static> {
  pub(crate) ftex: Mutex<BoxFuture<T>>,
  resolved: Mutex<Option<Box<T>>>,
}

impl<T: Send> Task<T> {
  #[inline(always)]
  pub fn new(
    fut: impl Future<Output = Box<dyn Taskable + Send + Sync>> + 'static + Send,
  ) -> Task<dyn Taskable + Send + Sync> {
    Task {
      ftex: Mutex::new(Box::pin(fut)),
      resolved: Mutex::new(None),
    }
  }
}

impl<T: Send + Sync> Future for Task<T> {
  type Output = Result<Box<T>, TaskError>;
  fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    let Ok(mut rtex) = self.resolved.lock() else {
      return Poll::Ready(Err(TaskError));
    };

    let Some(resolved) = rtex.take() else {
      return Poll::Pending;
    };

    Poll::Ready(Ok(resolved))
  }
}

impl Wake for Task<dyn Taskable + Send + Sync> {
  #[inline]
  fn wake(self: Arc<Self>) {
    todo!()
  }
}
