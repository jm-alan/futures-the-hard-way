use std::{
  error::Error,
  fmt::Display,
  pin::Pin,
  sync::{Arc, Mutex, MutexGuard},
  task::{Context, Poll, Wake, Waker},
};

use crate::types::{BoxFuture, TaskWrappable};

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

#[derive(Clone)]
pub struct Task<T: ?Sized + Send + 'static> {
  state: Arc<TaskState<T>>,
}

struct TaskState<T: ?Sized + Send + 'static> {
  fut: Mutex<BoxFuture<T>>,
  resolved: Mutex<Option<Box<T>>>,
  waker: Mutex<Option<Waker>>,
}

impl<T: Send> Task<T> {
  #[inline(always)]
  pub fn new(f: impl Future<Output = Box<TaskWrappable>> + 'static + Send) -> Task<TaskWrappable> {
    Task {
      state: Arc::new(TaskState {
        fut: Mutex::new(Box::pin(f)),
        resolved: Mutex::new(None),
        waker: Mutex::new(None),
      }),
    }
  }
}

impl Future for Task<TaskWrappable> {
  type Output = Result<Box<TaskWrappable>, TaskError>;
  fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    let Ok(mut rtex) = self.state.resolved.lock() else {
      return Poll::Ready(Err(TaskError));
    };

    let state = self.state.clone();
    let new_task = Task { state };
    let waker = Waker::from(Arc::new(new_task));
    let inner_cx = &mut Context::from_waker(&waker);

    let Some(resolved) = rtex.take() else {
      let Ok(mut ftex) = self.state.fut.lock() else {
        return Poll::Ready(Err(TaskError));
      };
      if let Poll::Ready(res) = ftex.as_mut().poll(inner_cx) {
        return Poll::Ready(Ok(res));
      };

      let Ok(mut wtex) = self.state.waker.lock() else {
        return Poll::Ready(Err(TaskError));
      };
      if let Some(old) = wtex.replace(cx.waker().clone()) {
        drop(old);
      };
      return Poll::Pending;
    };

    Poll::Ready(Ok(resolved))
  }
}

impl Wake for Task<TaskWrappable> {
  #[inline]
  fn wake(self: Arc<Self>) {
    let Ok(mut ftex) = self.state.fut.lock() else {
      return;
    };

    let waker = Waker::from(self.clone());
    let cx = &mut Context::from_waker(&waker);
    let Poll::Ready(resolved) = ftex.as_mut().poll(cx) else {
      return;
    };
    let Ok(mut rtex) = self.state.resolved.lock() else {
      return;
    };
    rtex.replace(resolved);

    let Ok(mut wtex) = self.state.waker.lock() else {
      return;
    };
    let Some(waker) = wtex.take() else {
      return;
    };
    waker.wake();
  }
}
