use std::{
  pin::Pin,
  sync::{Arc, Mutex},
};

use super::{TaskState, Taskable};

pub(crate) struct InnerTask<T>
where
  T: Taskable,
{
  pub(crate) fut: Mutex<Pin<Box<dyn Future<Output = T>>>>,
  pub(crate) state: Arc<TaskState<T>>,
}
