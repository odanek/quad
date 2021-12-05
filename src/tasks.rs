mod task;
mod task_pool;
mod usages;

pub use task::Task;
pub use task_pool::{TaskPool, TaskPoolBuilder};
pub use usages::IoTaskPool;

pub fn logical_core_count() -> usize {
    num_cpus::get()
}

pub fn physical_core_count() -> usize {
    num_cpus::get_physical()
}
