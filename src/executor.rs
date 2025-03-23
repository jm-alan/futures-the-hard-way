use std::{
  collections::VecDeque,
  sync::{Arc, Mutex},
};

use crate::task::Task;

pub struct Executor {
  tasks: Mutex<VecDeque<Task>>,
}

impl Executor {
  pub fn new() -> Self {
    Self {
      tasks: Mutex::new(VecDeque::new()),
    }
  }

  pub fn arc_new() -> Arc<Self> {
    Arc::new(Self::new())
  }

  pub fn run(&mut self) {
    while let Ok(task_queue) = self.tasks.lock() {}
  }
}
