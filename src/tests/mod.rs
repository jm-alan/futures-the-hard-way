use crate::executor::Executor;

#[test]
fn timer() {
  Executor::main(|_| async {
    println!("Async hello! :)");
  });
}
