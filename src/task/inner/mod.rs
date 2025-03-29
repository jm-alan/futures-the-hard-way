use std::{
  pin::Pin,
  sync::{Arc, Mutex},
  task::{Context, Poll, Wake, Waker},
};

use crate::executor::ExecTasks;

use super::Taskable;

#[derive(Clone)]
pub(crate) struct InnerTask {
  pub(crate) fut: Arc<Mutex<Pin<Box<dyn Future<Output = ()> + Send>>>>,
  pub(crate) task_handle: Arc<Mutex<ExecTasks>>,
}

impl InnerTask {
  pub(crate) fn new(
    task_handle: Arc<Mutex<ExecTasks>>,
    f: impl Future<Output = ()> + Taskable,
  ) -> Self {
    Self {
      fut: Arc::new(Mutex::new(Box::pin(f))),
      task_handle,
    }
  }
  pub(crate) fn step(self: Arc<Self>) {
    println!("Successfully called lock on self");
    let Ok(mut ftex) = self.fut.lock() else {
      return;
    };
    let Ok(mut ttex) = self.task_handle.lock() else {
      return;
    };

    let waker = Waker::from(self.clone());
    let cx = &mut Context::from_waker(&waker);

    let Poll::Ready(_) = ftex.as_mut().poll(cx) else {
      ttex.push_back(self.clone());
      return;
    };
  }
}

impl Wake for InnerTask {
  fn wake(self: Arc<Self>) {
    self.step()
  }
}
