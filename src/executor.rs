use core::future::Future;
use std::{
  collections::VecDeque,
  pin::Pin,
  sync::{Arc, Mutex, MutexGuard},
  task::{Context, Poll, Wake, Waker},
  thread,
  time::Instant,
};

use crate::{
  task::{Task, Taskable},
  timer::TIMER_RESOLUTION,
  types::TaskWrappable,
};

pub struct Executor {
  tasks: Mutex<VecDeque<Pin<Arc<Mutex<Task<TaskWrappable>>>>>>,
  last_checked_timers: Mutex<Instant>,
}

impl Executor {
  fn new() -> Arc<Self> {
    Arc::new(Self {
      tasks: Mutex::new(VecDeque::new()),
      last_checked_timers: Mutex::new(Instant::now()),
    })
  }

  #[inline(always)]
  pub fn main<T>(mn: impl FnOnce(Arc<Executor>) -> Box<(dyn Future<Output = T> + Send + 'static)>)
  where
    T: Send + Sync + 'static,
  {
    println!("");
    let mut exe = Self::new();
    let Ok(_) = exe.spawn(Box::into_pin(mn(exe.clone()))) else {
      panic!("Executor internal mutex poisoned before initialization.");
    };

    let Ok(mut tq) = exe.tasks.lock() else {
      panic!("Executor internal mutex poisoned before initialization.");
    };

    while let Some(task) = tq.pop_front() {
      println!("Popped task; assessing");
      let Ok(ttex) = task.lock() else {
        println!("Task mutex poisoned; discarding");
        continue;
      };

      println!("Pinning task");
      let mut pinned = Pin::new(ttex);

      let waker_clone = exe.clone();
      let waker = Waker::from(waker_clone);
      let cx = &mut Context::from_waker(&waker);

      println!("Polling");
      let Poll::Ready(_) = pinned.as_mut().poll(cx) else {
        println!("Task unresolved; pushing back and continuing");
        tq.push_back(task.clone());
        let Ok(mut lctex) = exe.last_checked_timers.lock() else {
          panic!("Executor timer management mutex poisoned.");
        };
        *lctex = Instant::now();

        println!("Attempting to stall for {:?}", TIMER_RESOLUTION);
        let Some(dont_open_til_christmas) = lctex.checked_add(TIMER_RESOLUTION) else {
          panic!("Timer increment overflow.");
        };
        while Instant::now() < dont_open_til_christmas {
          println!("Parking");
          thread::park_timeout(TIMER_RESOLUTION);
        }
        continue;
      };
    }
  }

  #[inline(always)]
  pub fn spawn<T>(
    self: &mut Arc<Self>,
    f: impl Future<Output = T> + Send + 'static,
  ) -> Result<Pin<Arc<Task<T>>>, ()>
  where
    T: Send + Sync,
  {
    let Ok(ref mut tasks) = self.tasks.lock() else {
      return Err(());
    };

    let arc_task = Arc::pin(Mutex::new(Task::<T>::new(async {
      Box::new(f.await) as Box<TaskWrappable>
    })));

    tasks.push_back(arc_task.clone());

    Ok(unsafe { (*((&arc_task) as *const _ as *const Pin<Arc<Task<T>>>)).clone() })
  }
}

impl Wake for Executor {
  fn wake(self: Arc<Self>) {}
}
