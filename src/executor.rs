use std::sync::{
  Arc,
  mpsc::{Receiver, sync_channel},
};

use crate::{spawner::Spawner, task::Task};

pub struct Executor {
  queue: Receiver<Arc<Task>>,
}

impl Executor {
  pub fn exec_pair() -> (Executor, Spawner) {
    let (sender, queue) = sync_channel(10_000);

    (Executor { queue }, Spawner::new(sender))
  }

  pub fn run(&self) {
    while let Ok(task) = self.queue.recv() {
      task.tick();
    }
  }
}
