use std::sync::{Arc, mpsc::SyncSender};

use crate::task::Task;

#[derive(Debug, Clone)]
pub struct Spawner {
  sender: SyncSender<Arc<Task>>,
}

impl Spawner {
  pub fn new(sender: SyncSender<Arc<Task>>) -> Self {
    Self { sender }
  }

  pub fn spawn(&self, f: impl Future<Output = ()> + 'static + Send) {
    match self
      .sender
      .send(Arc::new(Task::new(f, self.sender.clone())))
    {
      Err(err) => println!("Failed to spawn task: {err:?}"),
      _ => {},
    }
  }
}
