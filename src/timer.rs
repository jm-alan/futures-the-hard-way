use std::{
  error::Error,
  fmt::{self, Display, Formatter},
  pin::Pin,
  task::{Context, Poll},
  thread,
  time::{Duration, Instant},
};

pub(crate) const TIMER_RESOLUTION: Duration = Duration::new(0, 499_999);

pub struct Timer {
  target: Instant,
}

#[derive(Debug)]
pub struct TimerError(String);

impl Display for TimerError {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    f.write_str(&format!("Timer failed unexpectedly: {}", self.0))
  }
}

impl Error for TimerError {}

impl Future for Timer {
  type Output = Result<(), TimerError>;
  fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    let poll_start = Instant::now();

    if poll_start >= self.target {
      println!("Timer complete");
      Poll::Ready(Ok(()))
    } else {
      println!("Timer still has {:?} to go", self.target - poll_start);
      let sleep_until = poll_start + TIMER_RESOLUTION;
      while Instant::now() < sleep_until {
        thread::park_timeout(sleep_until - Instant::now());
      }
      cx.waker().clone().wake();

      Poll::Pending
    }
  }
}

impl Timer {
  #[inline(always)]
  pub fn new(dur: Duration) -> Self {
    Self {
      target: Instant::now() + dur,
    }
  }
}
