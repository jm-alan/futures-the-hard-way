use std::{
  collections::HashSet,
  sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering},
    mpsc::channel,
  },
  thread::{self, JoinHandle},
};

use crate::{
  Task,
  task::{InnerTask, TaskState, Taskable},
};

use super::{SHARED_EXEC, SHARED_HANDLE, SpawnError, TaskSender, WaitingTaskHandle};

pub struct SpawnHandle {
  pool_size: usize,
  send_pool: Vec<TaskSender>,
  join_handles: Vec<JoinHandle<()>>,
  waiting_task_handle: WaitingTaskHandle,
  next_id: AtomicUsize,
  next_idx: AtomicUsize,
}

impl SpawnHandle {
  #[inline(always)]
  pub(crate) fn new(
    send_pool: Vec<TaskSender>,
    join_handles: Vec<JoinHandle<()>>,
    waiting_task_handle: WaitingTaskHandle,
  ) -> Self {
    Self {
      pool_size: send_pool.len(),
      send_pool,
      join_handles,
      waiting_task_handle,
      next_id: AtomicUsize::new(0),
      next_idx: AtomicUsize::new(0),
    }
  }

  pub(crate) fn current() -> Arc<SpawnHandle> {
    SHARED_HANDLE
      .get_or_init(|| {
        let exe = SHARED_EXEC.wait();
        let channel_pairs = (0..exe.threads)
          .map(|_| {
            let (s, r) = channel::<Option<Arc<InnerTask>>>();
            (s, r)
          })
          .collect::<Vec<_>>();

        let mut send_pool = vec![];
        let mut recv_pool = vec![];

        for (sender, receiver) in channel_pairs {
          send_pool.push(sender);
          recv_pool.push(Some(receiver));
        }

        let join_handles = (1..exe.threads)
          .map(|idx| {
            let rec = unsafe { recv_pool[idx].take().unwrap_unchecked() };
            thread::spawn(move || {
              while let Ok(Some(inner)) = rec.recv() {
                inner.step();
              }
            })
          })
          .collect::<Vec<_>>();

        Arc::new(SpawnHandle::new(
          send_pool,
          join_handles,
          Arc::new(Mutex::new(HashSet::new())),
        ))
      })
      .clone()
  }

  #[inline]
  pub fn spawn<T>(&self, f: impl Future<Output = T> + Taskable) -> Result<Task<T>, SpawnError>
  where
    T: Taskable,
  {
    let state = Arc::new(Mutex::new(TaskState::new()));
    let next_id = self.next_id.fetch_add(1, Ordering::SeqCst);
    let send_idx = unsafe {
      self
        .next_idx
        .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |prev| {
          Some((prev + 1) % self.pool_size)
        })
        .unwrap_unchecked()
    };

    let task = Task::new(next_id, state.clone());
    let Ok(_) = self.send_pool[send_idx].send(Some(Arc::new(InnerTask::new(
      next_id,
      self.send_pool[send_idx].clone(),
      self.waiting_task_handle.clone(),
      async move {
        let res = f.await;
        state
          .clone()
          .lock()
          .expect(&format!("Task<{next_id}> panicked during execution.",))
          .set(res);
      },
    )))) else {
      return Err(SpawnError::Dropped);
    };

    Ok(task)
  }

  pub(crate) fn join_all(&mut self) {
    while let Some(handle) = self.join_handles.pop() {
      handle.join();
    }
  }

  #[inline(always)]
  pub(crate) fn cancel_all(&self) {
    for sender in &self.send_pool {
      _ = sender.send(None);
    }
  }
}
