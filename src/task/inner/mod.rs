use std::{
  fmt::Debug,
  hash::Hash,
  pin::Pin,
  sync::{Arc, Mutex},
  task::{Context, Poll, Wake, Waker},
};

use crate::executor::{TaskSender, WaitingTaskHandle};

use super::Taskable;

#[derive(Clone)]
pub(crate) struct InnerTask {
  id: usize,
  fut: Arc<Mutex<Pin<Box<dyn Future<Output = ()> + Send>>>>,
  task_sender: TaskSender,
  waiting_task_handle: WaitingTaskHandle,
}

impl Debug for InnerTask {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(&format!("InnerTask<{}>", self.id))
  }
}

impl InnerTask {
  #[inline(always)]
  pub(crate) fn new(
    id: usize,
    task_sender: TaskSender,
    waiting_task_handle: WaitingTaskHandle,
    f: impl Future<Output = ()> + Taskable,
  ) -> Self {
    Self {
      id,
      fut: Arc::new(Mutex::new(Box::pin(f))),
      task_sender,
      waiting_task_handle,
    }
  }

  pub(crate) fn step(self: Arc<Self>) {
    let Ok(mut ftex) = self.fut.lock() else {
      return;
    };
    let waker = Waker::from(self.clone());
    let cx = &mut Context::from_waker(&waker);

    match ftex.as_mut().poll(cx) {
      Poll::Pending => {
        let Ok(mut dtex) = self.waiting_task_handle.lock() else {
          return;
        };
        dtex.insert(self.clone());
      },
      _ => {},
    };
  }
}

impl Wake for InnerTask {
  fn wake(self: Arc<Self>) {
    let Ok(mut dtex) = self.waiting_task_handle.lock() else {
      return;
    };

    dtex.remove(&self);

    _ = self.task_sender.send(Some(self.clone()));
  }
}

impl PartialEq for InnerTask {
  fn eq(&self, other: &Self) -> bool {
    self.id == other.id
  }
}

impl Eq for InnerTask {}
impl Hash for InnerTask {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    state.write_usize(self.id);
  }
}
