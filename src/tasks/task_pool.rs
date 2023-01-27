use std::{
    future::Future,
    sync::Arc,
    thread::{self, available_parallelism, JoinHandle},
};

use futures_lite::future;

use super::Task;

// use crate::Task;

#[derive(Debug, Default, Clone)]
pub struct TaskPoolBuilder {
    num_threads: Option<usize>,
    stack_size: Option<usize>,
    thread_name: Option<String>,
}

impl TaskPoolBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn num_threads(mut self, num_threads: usize) -> Self {
        self.num_threads = Some(num_threads);
        self
    }

    pub fn stack_size(mut self, stack_size: usize) -> Self {
        self.stack_size = Some(stack_size);
        self
    }

    pub fn thread_name(mut self, thread_name: String) -> Self {
        self.thread_name = Some(thread_name);
        self
    }

    pub fn build(self) -> TaskPool {
        TaskPool::new_internal(
            self.num_threads,
            self.stack_size,
            self.thread_name.as_deref(),
        )
    }
}

#[derive(Debug)]
struct TaskPoolInner {
    threads: Vec<JoinHandle<()>>,
    shutdown_tx: async_channel::Sender<()>,
}

impl Drop for TaskPoolInner {
    fn drop(&mut self) {
        self.shutdown_tx.close();

        let panicking = thread::panicking();
        for join_handle in self.threads.drain(..) {
            let res = join_handle.join();
            if !panicking {
                res.expect("Task thread panicked while executing.");
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct TaskPool {
    executor: Arc<async_executor::Executor<'static>>,
    inner: Arc<TaskPoolInner>,
}

impl TaskPool {
    pub fn new() -> Self {
        TaskPoolBuilder::new().build()
    }

    fn new_internal(
        num_threads: Option<usize>,
        stack_size: Option<usize>,
        thread_name: Option<&str>,
    ) -> Self {
        let (shutdown_tx, shutdown_rx) = async_channel::unbounded::<()>();

        let executor = Arc::new(async_executor::Executor::new());

        let num_threads = num_threads.unwrap_or_else(|| available_parallelism().unwrap().get());

        let threads = (0..num_threads)
            .map(|i| {
                let ex = Arc::clone(&executor);
                let shutdown_rx = shutdown_rx.clone();

                let thread_name = if let Some(thread_name) = thread_name {
                    format!("{thread_name} ({i})")
                } else {
                    format!("TaskPool ({i})")
                };

                let mut thread_builder = thread::Builder::new().name(thread_name);

                if let Some(stack_size) = stack_size {
                    thread_builder = thread_builder.stack_size(stack_size);
                }

                thread_builder
                    .spawn(move || {
                        let shutdown_future = ex.run(shutdown_rx.recv());
                        // Use unwrap_err because we expect a Closed error
                        future::block_on(shutdown_future).unwrap_err();
                    })
                    .expect("Failed to spawn thread.")
            })
            .collect();

        Self {
            executor,
            inner: Arc::new(TaskPoolInner {
                threads,
                shutdown_tx,
            }),
        }
    }

    pub fn thread_num(&self) -> usize {
        self.inner.threads.len()
    }

    pub fn spawn<T>(&self, future: impl Future<Output = T> + Send + 'static) -> Task<T>
    where
        T: Send + 'static,
    {
        Task::new(self.executor.spawn(future))
    }
}

impl Default for TaskPool {
    fn default() -> Self {
        Self::new()
    }
}
