use std::time::{Duration, Instant};

use crate::{executor::Executor, timer::Timer};

#[test]
fn timer() {
  Executor::main(|_| {
    Box::new(async {
      let start = Instant::now();
      let Ok(_) = Timer::new(Duration::new(5, 0)).await else {
        panic!("Timer failed unexpectedly");
      };
      let end = Instant::now();
      println!("Timer took {:?}", end - start);
    })
  });
}
