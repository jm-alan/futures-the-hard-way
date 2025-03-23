use std::{
  collections::VecDeque,
  sync::{Arc, Mutex},
  task::{Context, Poll, Wake, Waker},
};

use crate::task::Task;

pub struct Executor {
  tasks: Mutex<VecDeque<Task>>,
}

impl Executor {
  fn new() -> Arc<Self> {
    Arc::new(Self {
      tasks: Mutex::new(VecDeque::new()),
    })
  }

  #[inline(always)]
  pub fn main<T>(mn: impl AsyncFnOnce(Arc<Executor>) -> T) -> T {
    let exe = Self::new();
    let mut f = Box::pin(mn(exe.clone()));

    let waker = Waker::from(exe);
    let mut cx = Context::from_waker(&waker);

    loop {
      match f.as_mut().poll(&mut cx) {
        Poll::Ready(result) => return result,
        _ => {},
      }
    }
  }

  #[inline(always)]
  pub fn spawn<A>(self: Arc<Self>, f: A) -> Result<(), ()>
  where
    A: AsyncFnOnce() + Send,
    A::CallOnceFuture: Send + 'static,
  {
    let Ok(ref mut tasks) = self.tasks.lock() else {
      return Err(());
    };

    tasks.push_back(Task::new(f()));

    Ok(())
  }
}

impl Wake for Executor {
  fn wake(self: Arc<Self>) {}
}
