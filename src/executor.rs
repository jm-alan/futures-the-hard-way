use std::{
  collections::VecDeque,
  sync::{Arc, Mutex},
  task::{Context, Poll, Wake, Waker},
};

use crate::task::{Task, Taskable};

pub struct Executor {
  tasks: Mutex<VecDeque<Arc<Task<dyn Taskable + Send + Sync>>>>,
}

impl Executor {
  fn new() -> Arc<Self> {
    Arc::new(Self {
      tasks: Mutex::new(VecDeque::new()),
    })
  }

  #[inline(always)]
  pub fn main<T>(mn: impl AsyncFnOnce(Arc<Executor>) -> T) {
    let exe = Self::new();
    let mut f = Box::pin(mn(exe.clone()));

    while let Ok(mut tq) = exe.tasks.lock() {
      while let Some(task) = tq.pop_front() {
        let waker_clone = task.clone();
        let Ok(mut ftex) = task.ftex.lock() else {
          continue;
        };

        let waker = Waker::from(waker_clone);
        let cx = &mut Context::from_waker(&waker);

        let Poll::Ready(_) = ftex.as_mut().poll(cx) else {
          drop(ftex);
          tq.push_back(task);
          return;
        };
      }
    }
  }

  #[inline(always)]
  pub fn spawn<T: Taskable>(
    self: Arc<Self>,
    f: impl Future<Output = T> + Send + 'static,
  ) -> Result<Arc<Task<T>>, ()>
  where
    T: Send + Sync,
  {
    let Ok(ref mut tasks) = self.tasks.lock() else {
      return Err(());
    };

    let arc_task = Arc::new(Task::<T>::new(async {
      Box::new(f.await) as Box<dyn Taskable + Send + Sync>
    }));

    tasks.push_back(arc_task.clone());

    Ok(unsafe { (*((&arc_task) as *const _ as *const Arc<Task<T>>)).clone() })
  }
}

impl Wake for Executor {
  fn wake(self: Arc<Self>) {}
}
