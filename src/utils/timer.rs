use std::time::{Duration, Instant};

use kqueue::{Event, EventData, EventFilter, Watcher};

#[derive(Debug, Clone, Copy)]
pub struct Timer {
  id: usize,
  dur: Duration,
}

impl Timer {
  pub(crate) fn new(id: usize, dur: Duration, watcher: &mut Watcher) -> Self {
    watcher.add_timer(id, dur);

    let timer = Self { id, dur };
  }
}
