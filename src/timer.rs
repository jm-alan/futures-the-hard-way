use std::{
  error::Error,
  fmt::{self, Display, Formatter},
  pin::Pin,
  sync::{Arc, Mutex},
  task::{Context, Poll, Waker},
  thread,
  time::Duration,
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

#[derive(Default)]
struct State {
  complete: bool,
  waker: Option<Waker>,
}

impl Future for Timer {
  type Output = Result<(), TimerError>;
  fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    let Ok(ref mut state) = self.state.lock() else {
      return Poll::Ready(Err(TimerError("Internal mutex poisoned".to_string())));
    };

    if state.complete {
      Poll::Ready(Ok(()))
    } else {
      state.waker = Some(cx.waker().clone());

      Poll::Pending
    }
  }
}

impl Timer {
  #[inline(always)]
  pub fn new(dur: Duration) -> Self {
    let state = Arc::new(Mutex::new(State::default()));

    let thread_state = state.clone();
    thread::spawn(move || {
      thread::park();
      thread::sleep(dur);
      let Ok(ref mut local) = thread_state.lock() else {
        return;
      };

      local.complete = true;
      if let Some(waker) = local.waker.take() {
        waker.wake();
      };
    });

    Self { state }
  }
}
