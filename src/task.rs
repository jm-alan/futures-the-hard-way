use std::{
  sync::{Arc, Mutex, mpsc::SyncSender},
  task::{Context, Wake, Waker},
};

use crate::types::BoxFuture;

pub struct Task {
  ftex: Mutex<Option<BoxFuture<'static, ()>>>,
  sender: SyncSender<Arc<Self>>,
}

impl Task {
  #[inline(always)]
  pub fn new(
    fut: impl Future<Output = ()> + 'static + Send,
    sender: SyncSender<Arc<Self>>,
  ) -> Self {
    Self {
      ftex: Mutex::new(Some(Box::pin(fut))),
      sender,
    }
  }

  pub fn tick(self: Arc<Self>) {
    let Ok(mut mfut) = self.ftex.lock() else {
      println!("Task internal mutex poisoned.");
      return;
    };

    let Some(mut f) = mfut.take() else {
      return;
    };

    let waker = Waker::from(self.clone());

    let cx = &mut Context::from_waker(&waker);
    if f.as_mut().poll(cx).is_pending() {
      *mfut = Some(f);
    }
  }
}

impl Wake for Task {
  #[inline]
  fn wake(self: Arc<Self>) {
    let sender = self.sender.clone();

    match sender.send(self) {
      Err(err) => println!("Failed to reschedule completed task: {err:?}"),
      _ => {},
    }
  }
}
