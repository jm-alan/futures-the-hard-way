use std::{
  error::Error,
  fmt::{self, Display, Formatter},
  pin::Pin,
  sync::{Arc, Mutex},
  task::{Context, Poll, Waker},
  thread,
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

const TIMER_RESOLUTION: Duration = Duration::new(0, 99999);

impl Timer {
  #[inline(always)]
  pub fn new(dur: Duration) -> Self {
    let now = Instant::now();
    let then = now + dur;
    let state = Arc::new(Mutex::new(State::default()));

    let thread_state = state.clone();
    thread::spawn(move || {
      while Instant::now() < then {
        thread::park_timeout(TIMER_RESOLUTION);
      }
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

/// TODO:
/// 1. Execution queue for complete tasks, sleep queue for incomplete tasks
/// 2. Executor-compatible tasks ask to be spawned on the executor instance, or for the executor to
/// spawn them directly
///     - Trait for something like "ExecutorSpawnable"
///     - Impl for Timer
/// 3. Timer should no longer be responsible for its own thread; whether the runtime is threaded is
/// an implementation detail at a higher level from, and therefore opaque to, the timer
/// implementation; instead, its `poll` should simply return `Pending` if its target Instant has not
/// yet come to pass
/// 4. Instead, executor will be responsible for polling the sleep queue, and if necessary, spinning
/// on a single timer no faster than TIMER_RESOLUTION
const FUCK_YOU_I_JUST_WANTED_TO_USE_DOC_COMMENT_SYNTAX_FOR_AUTO_RE_COMMENT_ON_NEWLINE: () = ();
