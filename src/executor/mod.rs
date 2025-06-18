mod spawn_handle;

use std::{
  collections::HashSet,
  error::Error,
  fmt::Display,
  sync::{
    Arc, Mutex,
    mpsc::{SyncSender, sync_channel},
  },
  task::{Context, Poll, Wake, Waker},
  thread::{self},
};

use crate::task::{InnerTask, Taskable};
pub use spawn_handle::SpawnHandle;

pub(crate) type TaskSender = SyncSender<Option<Arc<InnerTask>>>;
pub(crate) type WaitingTaskHandle = Arc<Mutex<HashSet<Arc<InnerTask>>>>;

pub struct Executor;

impl Executor {
  #[inline(always)]
  pub fn main<'exe, Main, FWrap, Result>(
    threads: usize,
    max_concurrent_tasks: usize,
    m: Main,
  ) -> Result
  where
    Main: (FnOnce(Arc<SpawnHandle>) -> FWrap) + Taskable,
    FWrap: Future<Output = Result> + Taskable,
    Result: Taskable,
  {
    let mut channel_pairs = (0..threads)
      .map(|_| {
        let (s, r) = sync_channel(max_concurrent_tasks / threads);
        (Some(s), Some(r))
      })
      .collect::<Vec<_>>();

    let mut recv_pool = (0..threads)
      .map(|idx| channel_pairs[idx].1.take())
      .collect::<Vec<_>>();

    let spawn_handle = Arc::new(SpawnHandle::new(
      (0..threads)
        .map(|idx| unsafe { channel_pairs[idx].0.take().unwrap_unchecked() })
        .collect(),
      Arc::new(Mutex::new(HashSet::new())),
    ));
    let fmain = m(spawn_handle.clone());
    let handle_post_main = spawn_handle.clone();
    let Ok(main_task) = spawn_handle.spawn(async move {
      let res = fmain.await;
      handle_post_main.cancel_all();
      return res;
    }) else {
      panic!("Executor sender/receiver pair closed before main began execution.");
    };

    let join_handles = (0..(threads - 1))
      .map(|idx| {
        let rec = unsafe {
          recv_pool
            .get_mut(idx)
            .unwrap_unchecked()
            .take()
            .unwrap_unchecked()
        };
        thread::spawn(move || {
          while let Ok(Some(inner)) = rec.recv() {
            inner.step();
          }
        })
      })
      .collect::<Vec<_>>();

    let final_receiver = unsafe {
      recv_pool
        .get_mut(threads - 1)
        .unwrap_unchecked()
        .take()
        .unwrap_unchecked()
    };

    while let Ok(Some(inner)) = final_receiver.recv() {
      inner.step();
    }
    for handle in join_handles {
      _ = handle.join();
    }

    let waker = Waker::from(Arc::new(Executor));
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
