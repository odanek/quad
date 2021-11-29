use std::{future::Future, pin::Pin};

pub type BoxedFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;
