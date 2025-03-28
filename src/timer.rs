use std::{
  error::Error,
  fmt::{self, Display, Formatter},
  pin::Pin,
  sync::{Arc, Mutex},
  task::{Context, Poll, Waker},
  time::{Duration, Instant},
};

pub struct Timer {
  state: Arc<Mutex<State>>,
}

#[derive(Debug)]
pub struct TimerError(String);

impl Display for TimerError {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    f.write_str(&format!("Timer failed unexpectedly: {}", self.0))
  }
}

impl Error for TimerError {}

struct State {
  target: Instant,
  waker: Option<Waker>,
}

impl Future for Timer {
  type Output = Result<(), TimerError>;
  fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    println!("Timer being polled");
    let Ok(ref mut state) = self.state.lock() else {
      return Poll::Ready(Err(TimerError("Internal mutex poisoned".to_string())));
    };

    let now = Instant::now();

    if now >= state.target {
      Poll::Ready(Ok(()))
    } else {
      println!("Timer still has {:?} to go", state.target - now);
      state.waker = Some(cx.waker().clone());

      Poll::Pending
    }
  }
}

pub const TIMER_RESOLUTION: Duration = Duration::new(0, 499_999);

impl Timer {
  #[inline(always)]
  pub fn new(dur: Duration) -> Self {
    let state = Arc::new(Mutex::new(State {
      target: Instant::now() + dur,
      waker: None,
    }));

    Self { state }
  }
}
