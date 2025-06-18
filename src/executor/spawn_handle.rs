use std::sync::{
  Arc, Mutex,
  atomic::{AtomicUsize, Ordering},
};

use crate::{
  Task,
  task::{InnerTask, TaskState, Taskable},
};

use super::{SpawnError, TaskSender, WaitingTaskHandle};

pub struct SpawnHandle {
  pool_size: usize,
  send_pool: Arc<[TaskSender]>,
  waiting_task_handle: WaitingTaskHandle,
  next_id: AtomicUsize,
  next_regi: AtomicUsize,
}

impl SpawnHandle {
  #[inline(always)]
  pub(crate) fn new(send_pool: Arc<[TaskSender]>, waiting_task_handle: WaitingTaskHandle) -> Self {
    Self {
      pool_size: send_pool.len(),
      send_pool,
      waiting_task_handle,
      next_id: AtomicUsize::new(0),
      next_regi: AtomicUsize::new(0),
    }
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
        .next_regi
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

  #[inline(always)]
  pub(crate) fn cancel_all(&self) {
    for sender in self.send_pool.iter() {
      _ = sender.send(None);
    }
  }
}
