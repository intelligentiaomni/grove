use core::cell::UnsafeCell;
use core::mem::MaybeUninit;
use core::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelError {
    Full,
    Empty,
    InvalidCapacity,
}

struct ChannelSlot<T: Copy> {
    value: UnsafeCell<MaybeUninit<T>>,
}

impl<T: Copy> ChannelSlot<T> {
    const fn uninit() -> Self {
        Self {
            value: UnsafeCell::new(MaybeUninit::uninit()),
        }
    }
}

unsafe impl<T: Copy + Send> Sync for ChannelSlot<T> {}

pub struct KernelChannel<T: Copy, const N: usize> {
    slots: [ChannelSlot<T>; N],
    head: AtomicUsize,
    tail: AtomicUsize,
}

impl<T: Copy, const N: usize> KernelChannel<T, N> {
    pub const fn new() -> Self {
        Self {
            slots: [const { ChannelSlot::uninit() }; N],
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
        }
    }

    pub fn push(&self, value: T) -> Result<(), ChannelError> {
        if N == 0 {
            return Err(ChannelError::InvalidCapacity);
        }

        let tail = self.tail.load(Ordering::Relaxed);
        let head = self.head.load(Ordering::Acquire);

        if tail.wrapping_sub(head) == N {
            return Err(ChannelError::Full);
        }

        let slot = tail % N;
        unsafe {
            (*self.slots[slot].value.get()).write(value);
        }
        self.tail.store(tail.wrapping_add(1), Ordering::Release);
        Ok(())
    }

    pub fn pop(&self) -> Result<T, ChannelError> {
        if N == 0 {
            return Err(ChannelError::InvalidCapacity);
        }

        let head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Acquire);

        if head == tail {
            return Err(ChannelError::Empty);
        }

        let slot = head % N;
        let value = unsafe { (*self.slots[slot].value.get()).assume_init_read() };
        self.head.store(head.wrapping_add(1), Ordering::Release);
        Ok(value)
    }

    pub fn len(&self) -> usize {
        let head = self.head.load(Ordering::Acquire);
        let tail = self.tail.load(Ordering::Acquire);
        tail.wrapping_sub(head)
    }

    pub fn capacity(&self) -> usize {
        N
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn is_full(&self) -> bool {
        self.len() == N
    }
}

impl<T: Copy, const N: usize> Default for KernelChannel<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelSchedulerAction {
    Continue {
        thread_id: u64,
    },
    Defer {
        thread_id: u64,
    },
    Fork {
        thread_id: u64,
        child_thread_id: u64,
    },
    Complete {
        thread_id: u64,
    },
}

impl KernelSchedulerAction {
    pub const fn thread_id(self) -> u64 {
        match self {
            Self::Continue { thread_id }
            | Self::Defer { thread_id }
            | Self::Fork { thread_id, .. }
            | Self::Complete { thread_id } => thread_id,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FiberState {
    Ready,
    Running,
    Deferred,
    Forked { child_thread_id: u64 },
    Complete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KernelFiber {
    pub thread_id: u64,
    pub state: FiberState,
    pub transitions: u64,
}

impl KernelFiber {
    pub const fn new(thread_id: u64) -> Self {
        Self {
            thread_id,
            state: FiberState::Ready,
            transitions: 0,
        }
    }

    pub fn apply_scheduler_action(&mut self, action: KernelSchedulerAction) {
        if action.thread_id() != self.thread_id {
            return;
        }

        self.state = match action {
            KernelSchedulerAction::Continue { .. } => FiberState::Running,
            KernelSchedulerAction::Defer { .. } => FiberState::Deferred,
            KernelSchedulerAction::Fork {
                child_thread_id, ..
            } => FiberState::Forked { child_thread_id },
            KernelSchedulerAction::Complete { .. } => FiberState::Complete,
        };
        self.transitions = self.transitions.wrapping_add(1);
    }
}

#[cfg(test)]
mod tests {
    use super::{ChannelError, FiberState, KernelChannel, KernelFiber, KernelSchedulerAction};

    #[test]
    fn ring_buffer_preserves_fifo_order_without_allocation() {
        let channel = KernelChannel::<u32, 2>::new();

        assert_eq!(channel.push(10), Ok(()));
        assert_eq!(channel.push(20), Ok(()));
        assert_eq!(channel.push(30), Err(ChannelError::Full));
        assert_eq!(channel.pop(), Ok(10));
        assert_eq!(channel.push(30), Ok(()));
        assert_eq!(channel.pop(), Ok(20));
        assert_eq!(channel.pop(), Ok(30));
        assert_eq!(channel.pop(), Err(ChannelError::Empty));
    }

    #[test]
    fn zero_capacity_channel_reports_invalid_capacity() {
        let channel = KernelChannel::<u8, 0>::new();

        assert_eq!(channel.push(1), Err(ChannelError::InvalidCapacity));
        assert_eq!(channel.pop(), Err(ChannelError::InvalidCapacity));
    }

    #[test]
    fn kernel_fiber_transitions_from_scheduler_actions() {
        let mut fiber = KernelFiber::new(7);

        fiber.apply_scheduler_action(KernelSchedulerAction::Continue { thread_id: 7 });
        assert_eq!(fiber.state, FiberState::Running);

        fiber.apply_scheduler_action(KernelSchedulerAction::Fork {
            thread_id: 7,
            child_thread_id: 11,
        });
        assert_eq!(
            fiber.state,
            FiberState::Forked {
                child_thread_id: 11
            }
        );

        fiber.apply_scheduler_action(KernelSchedulerAction::Complete { thread_id: 7 });
        assert_eq!(fiber.state, FiberState::Complete);
        assert_eq!(fiber.transitions, 3);
    }

    #[test]
    fn kernel_fiber_ignores_other_threads() {
        let mut fiber = KernelFiber::new(7);

        fiber.apply_scheduler_action(KernelSchedulerAction::Defer { thread_id: 8 });

        assert_eq!(fiber.state, FiberState::Ready);
        assert_eq!(fiber.transitions, 0);
    }
}
