use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub struct TaskError;

impl Display for TaskError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str("Task internal mutex poisoned")
  }
}

impl Error for TaskError {}
