use std::{
  fs::File,
  io::{Read, Write},
  time::Instant,
};

use crate::{Task, executor::Executor};

const THREADS: usize = 4;
const MAX_TASKS: usize = 1_000_000;

#[test]
fn fs() {
  let then = Instant::now();
  Executor::main(THREADS, MAX_TASKS, move |exe| async move {
    for _ in 0..100000 {
      let tasks: Vec<Task<()>> = (0..10)
        .map(|_| unsafe {
          exe
            .spawn(async move {
              let mut buffer = [0; 1024 * 8];
              let mut dev_urandom = File::open("/dev/urandom").unwrap();
              dev_urandom.read(&mut buffer).unwrap();
              let mut dev_null = File::create("/dev/null").unwrap();
              dev_null.write(&mut buffer).unwrap();
            })
            .unwrap_unchecked()
        })
        .collect();

      for task in tasks {
        unsafe { task.await.unwrap_unchecked() }
      }
    }
  });
  println!("Elapsed: {:?}", then.elapsed());
}
