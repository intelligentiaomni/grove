//! Execution scheduling.
//
//! Determines ordering and selection of tasks.

use super::task::Task;

/// Scheduler abstraction.
pub trait Scheduler {
    type Task: Task;

    /// Add a task to the schedule.
    fn submit(&mut self, task: Self::Task);

    /// Retrieve the next task to execute.
    fn next(&mut self) -> Option<Self::Task>;

    /// Check whether work remains.
    fn is_empty(&self) -> bool;
}


/// Simple FIFO scheduler.
#[derive(Debug, Default)]
pub struct FifoScheduler<T>
where
    T: Task,
{
    queue: std::collections::VecDeque<T>,
}

impl<T> FifoScheduler<T>
where
    T: Task,
{
    pub fn new() -> Self {
        Self {
            queue: std::collections::VecDeque::new(),
        }
    }
}

impl<T> Scheduler for FifoScheduler<T>
where
    T: Task,
{
    type Task = T;

    fn submit(&mut self, task: T) {
        self.queue.push_back(task);
    }

    fn next(&mut self) -> Option<T> {
        self.queue.pop_front()
    }

    fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
}