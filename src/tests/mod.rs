use std::{hint::black_box, time::Instant};

use crate::executor::Executor;

const THREADS: usize = 4;
const TASKS: usize = 1000;
const RUNS: usize = 1000;
const LOOPS_PER_TASK: usize = 100_000;
const MAX_TASKS: usize = 1_000_000;

#[test]
fn fs() {
  let then = Instant::now();
  Executor::main(move || async move {
    for _ in 0..RUNS {
      let tasks = (0..TASKS)
        .map(|_| unsafe {
          handle
            .spawn(async move {
              for _ in 0..LOOPS_PER_TASK {
                black_box(());
              }
            })
            .unwrap_unchecked()
        })
        .collect::<Vec<_>>();

      for task in tasks {
        unsafe { task.await.unwrap_unchecked() }
      }
    }
  });
  println!("Elapsed: {:?}", then.elapsed());
}
