use std::{
  collections::VecDeque,
  error::Error,
  fmt::Display,
  sync::{Arc, Mutex},
  task::{Context, Poll, Wake, Waker},
  time::Instant,
};

use crate::task::{InnerTask, Task, TaskState, Taskable};

pub(crate) type ExecTasks = VecDeque<Arc<InnerTask>>;

pub struct Executor {
  tasks: Arc<Mutex<ExecTasks>>,
  last_checked_timers: Arc<Mutex<Instant>>,
}

impl Executor {
  fn new() -> Self {
    Self {
      tasks: Arc::new(Mutex::new(VecDeque::new())),
      last_checked_timers: Arc::new(Mutex::new(Instant::now())),
    }
  }

  #[inline(always)]
  pub fn main<Main, FWrap, Result>(m: Main) -> Result
  where
    Main: FnOnce(&Executor) -> FWrap,
    FWrap: IntoFuture<Output = Result>,
    FWrap::IntoFuture: Taskable,
    Result: Taskable,
  {
    let exe = Self::new();

    let Ok(main_task) = exe.spawn(m(&exe).into_future()) else {
      panic!("Executor internal mutex poisoned before initialization.");
    };

    loop {
      let Ok(mut ttex) = exe.tasks.lock() else {
        panic!("Executor internal mutex poisoned; carrier thread must have panicked unexpectedly.");
      };
      let Some(inner) = ttex.pop_front() else {
        break;
      };
      // Unlock the mutex to prevent reentrant deadlock on inner.wake and permit a multi-threaded implementation in the future
      drop(ttex);
      inner.step();
    }

    let arc_exe = Arc::new(exe);

    loop {
      let waker = Waker::from(arc_exe.clone());
      let cx = &mut Context::from_waker(&waker);
      let mut pinned = Box::pin(main_task.clone());
      match pinned.as_mut().poll(cx) {
        Poll::Ready(Ok(res)) => return res,
        Poll::Ready(Err(err)) => panic!("{err:?}"),
        _ => {},
      }
    }
  }

  pub fn spawn<T>(&self, f: impl Future<Output = T> + Taskable) -> Result<Task<T>, SpawnError>
  where
    T: Taskable,
  {
    let Ok(mut ttex) = self.tasks.lock() else {
      return Err(SpawnError::Poisoned);
    };
    let state = Arc::new(Mutex::new(TaskState::new()));
    let cloned = state.clone();
    let inner = InnerTask::new(self.tasks.clone(), async move {
      let concrete = f.await;
      let Ok(mut ctex) = cloned.lock() else {
        return;
      };
      ctex.set(concrete);
    });

    ttex.push_back(Arc::new(inner));

    Ok(Task::new(state))
  }
}

impl Wake for Executor {
  fn wake(self: Arc<Self>) {}
}

#[derive(Debug)]
pub enum SpawnError {
  Poisoned,
}

impl Display for SpawnError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(match self {
      SpawnError::Poisoned => "Executor internal mutex poisoned. This is unrecoverable.",
    })
  }
}

impl Error for SpawnError {}
