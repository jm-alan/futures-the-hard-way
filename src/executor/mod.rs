mod spawn_handle;

use std::{
  collections::HashSet,
  error::Error,
  fmt::Display,
  num::NonZero,
  sync::{Arc, Mutex, OnceLock, mpsc::Sender},
  task::{Context, Poll, Wake, Waker},
  thread::{self, available_parallelism},
};

use crate::task::{InnerTask, Taskable};
pub use spawn_handle::SpawnHandle;

pub(crate) type TaskSender = Sender<Option<Arc<InnerTask>>>;
pub(crate) type WaitingTaskHandle = Arc<Mutex<HashSet<Arc<InnerTask>>>>;

static SHARED_EXEC: OnceLock<Executor> = OnceLock::new();
static SHARED_HANDLE: OnceLock<Arc<SpawnHandle>> = OnceLock::new();

pub struct Executor {
  threads: usize,
}

impl Default for Executor {
  fn default() -> Self {
    Executor {
      threads: available_parallelism()
        .unwrap_or(NonZero::new(1).unwrap())
        .into(),
    }
  }
}

impl Executor {
  /// Entrypoint function. Should only be called once in a given program.
  #[inline(always)]
  pub fn main<Main, FWrap, Result>(m: Main) -> Result
  where
    Main: (FnOnce() -> FWrap) + Taskable,
    FWrap: Future<Output = Result> + Taskable,
    Result: Taskable,
  {
    SHARED_EXEC.get_or_init(|| Self::default()).run(m)
  }
  #[inline(always)]
  pub fn run<Main, FWrap, Result>(&self, m: Main) -> Result
  where
    Main: (FnOnce() -> FWrap) + Taskable,
    FWrap: Future<Output = Result> + Taskable,
    Result: Taskable,
  {
    let spawn_handle = SpawnHandle::current();
    let handle_post_main = spawn_handle.clone();
    let Ok(main_task) = spawn_handle.spawn(async move {
      let res = m().await;
      handle_post_main.cancel_all();
      return res;
    }) else {
      panic!("Executor sender/receiver pair closed before main began execution.");
    };

    let core_receiver = unsafe { recv_pool[0].take().unwrap_unchecked() };

    while let Ok(Some(inner)) = core_receiver.recv() {
      inner.step();
    }
    spawn_handle.join_all();

    let waker = Waker::from(Arc::new(self));
    let context = &mut Context::from_waker(&waker);
    let mut pinned = Box::pin(main_task);
    let Poll::Ready(Ok(res)) = pinned.as_mut().poll(context) else {
      panic!(
        "Carrier thread for future containing execution of `main` panicked, or otherwise failed to persist in task list."
      );
    };

    return res;
  }
}

impl Wake for Executor {
  fn wake(self: Arc<Self>) {}
}

#[derive(Debug)]
pub enum SpawnError {
  Dropped,
}

impl Display for SpawnError {
  #[inline(always)]
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(match self {
      SpawnError::Dropped => "Attempted to use spawn handle after executor was dropped.",
    })
  }
}

impl Error for SpawnError {}
