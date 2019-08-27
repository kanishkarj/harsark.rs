#![no_std]

#[macro_use]
extern crate lazy_static;

mod kernel;

pub mod event_manager;
mod helper;
mod interrupt_handlers;
pub mod messaging;
pub mod resources;
pub mod sync;
mod task_manager;

use crate::errors::KernelError;
use core::fmt;

pub mod tasks {
    pub use crate::task_manager::create_task;
    pub use crate::task_manager::init;
    pub use crate::task_manager::release_tasks;
    pub use crate::task_manager::start_kernel;
    pub use crate::task_manager::task_exit;
    pub use crate::task_manager::TaskId;
}

mod config {
    pub const MAX_TASKS: usize = 32;
    pub const MAX_RESOURCES: usize = 32;
    pub const SYSTICK_INTERRUPT_INTERVAL: u32 = 15_000;
    pub const SEMAPHORE_COUNT: usize = 32;
    pub const MCB_COUNT: usize = 32;
    pub const MAX_BUFFER_SIZE: usize = 32;
    pub const EVENT_NO: usize = 32;
    pub const EVENT_INDEX_TABLE_COUNT: usize = 8;
    pub const MAX_STACK_SIZE: usize = 256;
}

pub mod errors {
    pub enum KernelError {
        BufferOverflow,
        NotFound,
        StackTooSmall,
        DoesNotExist,
        LimitExceeded,
    }
}

impl fmt::Debug for KernelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            KernelError::DoesNotExist => write!(f, "DoesNotExist"),
            KernelError::BufferOverflow => write!(f, "BufferOverflow"),
            KernelError::NotFound => write!(f, "NotFound"),
            KernelError::StackTooSmall => write!(f, "StackTooSmall"),
            KernelError::LimitExceeded => write!(f, "LimitExceeded"),
        }
    }
}
