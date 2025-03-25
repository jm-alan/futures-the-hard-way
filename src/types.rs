use std::pin::Pin;

use crate::task::Taskable;

pub(crate) type BoxFuture<T> = Pin<Box<dyn Future<Output = Box<T>> + Send + 'static>>;
pub(crate) type TaskWrappable = dyn Taskable + Send + Sync + 'static;
