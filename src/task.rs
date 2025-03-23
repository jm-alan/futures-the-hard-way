use std::{
  sync::{Arc, Mutex},
  task::{Context, Wake, Waker},
};

use crate::types::BoxFuture;

pub struct Task {
  ftex: Mutex<Option<BoxFuture<'static, ()>>>,
}

impl Task {
  #[inline(always)]
  pub fn new(fut: impl Future<Output = ()> + 'static + Send) -> Self {
    Self {
      ftex: Mutex::new(Some(Box::pin(fut))),
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
    todo!()
  }
}
