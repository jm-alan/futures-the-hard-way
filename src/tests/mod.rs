use std::time::{Duration, Instant};

use crate::{executor::Executor, timer::Timer};

#[test]
fn timer() {
  let (executor, spawner) = Executor::exec_pair();

  spawner.spawn(async {
    let then = Instant::now();
    let Ok(_) = Timer::new(Duration::new(60, 0)).await else {
      println!("Timer failed!");
      return;
    };
    let now = Instant::now();
    let diff = now - then;
    println!("Before timer: {then:?}");
    println!("After timer: {now:?}");
    println!("Diff: {diff:?}");
  });

  drop(spawner);

  executor.run();
}
