use std::thread::available_parallelism;

use crate::{
    ecs::World,
    tasks::{IoTaskPool, TaskPoolBuilder},
};

#[derive(Clone)]
pub struct TaskPoolThreadAssignmentPolicy {
    pub min_threads: usize,
    pub max_threads: usize,
    pub percent: f32,
}

impl TaskPoolThreadAssignmentPolicy {
    fn get_number_of_threads(&self, remaining_threads: usize, total_threads: usize) -> usize {
        assert!(self.percent >= 0.0);
        let mut desired = (total_threads as f32 * self.percent).round() as usize;
        desired = desired.min(remaining_threads);
        desired.clamp(self.min_threads, self.max_threads)
    }
}

#[derive(Clone)]
pub struct TaskPoolOptions {
    pub min_total_threads: usize,
    pub max_total_threads: usize,
    pub io: TaskPoolThreadAssignmentPolicy,
    // pub async_compute: TaskPoolThreadAssignmentPolicy,
    // pub compute: TaskPoolThreadAssignmentPolicy,
}

impl Default for TaskPoolOptions {
    fn default() -> Self {
        Self {
            min_total_threads: 1,
            max_total_threads: std::usize::MAX,
            io: TaskPoolThreadAssignmentPolicy {
                min_threads: 1,
                max_threads: 4,
                percent: 0.25,
            },
            // async_compute: TaskPoolThreadAssignmentPolicy {
            //     min_threads: 1,
            //     max_threads: 4,
            //     percent: 0.25,
            // },
            // compute: TaskPoolThreadAssignmentPolicy {
            //     min_threads: 1,
            //     max_threads: std::usize::MAX,
            //     percent: 1.0,
            // },
        }
    }
}

impl TaskPoolOptions {
    pub fn with_num_threads(thread_count: usize) -> Self {
        Self {
            min_total_threads: thread_count,
            max_total_threads: thread_count,
            ..Default::default()
        }
    }

    pub fn create_pools(&self, world: &mut World) {
        let total_threads = available_parallelism()
            .unwrap()
            .get()
            .clamp(self.min_total_threads, self.max_total_threads);
        log::trace!("Assigning {} cores to task pools", total_threads);

        let remaining_threads = total_threads;

        let io_threads = self
            .io
            .get_number_of_threads(remaining_threads, total_threads);

        log::trace!("IO Threads: {}", io_threads);
        // remaining_threads = remaining_threads.saturating_sub(io_threads);

        world.insert_resource(IoTaskPool(
            TaskPoolBuilder::default()
                .num_threads(io_threads)
                .thread_name("IO Task Pool".to_string())
                .build(),
        ));

        // let async_compute_threads = self
        //     .async_compute
        //     .get_number_of_threads(remaining_threads, total_threads);

        // log::trace!("Async Compute Threads: {}", async_compute_threads);
        // remaining_threads = remaining_threads.saturating_sub(async_compute_threads);

        // world.insert_resource(AsyncComputeTaskPool(
        //     TaskPoolBuilder::default()
        //         .num_threads(async_compute_threads)
        //         .thread_name("Async Compute Task Pool".to_string())
        //         .build(),
        // ));

        // let compute_threads = self
        //     .compute
        //     .get_number_of_threads(remaining_threads, total_threads);

        // log::trace!("Compute Threads: {}", compute_threads);
        // world.insert_resource(ComputeTaskPool(
        //     TaskPoolBuilder::default()
        //         .num_threads(compute_threads)
        //         .thread_name("Compute Task Pool".to_string())
        //         .build(),
        // ));
    }
}
