use crate::KernelError;

use crate::priv_execute;
use crate::system::task_manager::*;
use crate::utils::arch::svc_call;
use cortex_m::peripheral::syst::SystClkSource;

use cortex_m::interrupt::{Mutex, free as execute_critical};
use core::cell::RefCell;

use cortex_m::Peripherals;

use crate::system::types::{BooleanVector, TaskId};
use crate::utils::arch::is_privileged;

static empty_task: TaskControlBlock = TaskControlBlock { sp: 0 };

// GLOBALS:
pub static mut all_tasks: Scheduler = Scheduler::new();
// end GLOBALS

pub static os_curr_task_id: Mutex<RefCell<usize>> = Mutex::new(RefCell::new(0));
pub static os_next_task_id: Mutex<RefCell<usize>> = Mutex::new(RefCell::new(0));

/// Initialize the switcher system
pub fn init(is_preemptive: bool) {
    execute_critical(|_| unsafe { all_tasks.init(is_preemptive) })
}

// The below section just sets up the timer and starts it.
pub fn start_kernel(peripherals: &mut Peripherals, tick_interval: u32) -> Result<(), KernelError> {
    priv_execute!({
        let syst = &mut peripherals.SYST;
        syst.set_clock_source(SystClkSource::Core);
        syst.set_reload(tick_interval);
        syst.enable_counter();
        syst.enable_interrupt();

        execute_critical(|_| unsafe { all_tasks.start_kernel() });
        preempt()
    })
}

pub fn create_task<T: Sized>(
    priority: TaskId,
    stack: &mut [u32],
    handler_fn: fn(&T) -> !,
    param: &T,
) -> Result<(), KernelError>
where
    T: Sync,
{
    priv_execute!({
        execute_critical(|_| unsafe {
            all_tasks.create_task(priority as usize, stack, handler_fn, param)
        })
    })
}

pub fn schedule() {
    if is_privileged() == true {
        preempt();
    } else {
        svc_call();
    }
}

pub fn preempt() -> Result<(), KernelError> {
    execute_critical(|_| {
        let handler = unsafe { &mut all_tasks };
        let next_tid = handler.get_next_tid();
        let curr_tid = handler.curr_tid as TaskId;
        if handler.is_running {
            if curr_tid != next_tid {
                context_switch(curr_tid as usize, next_tid as usize);
            }
        }
        return Ok(());
    })
}

fn context_switch(curr: usize, next: usize) {
    execute_critical(|cs_token| {
        let handler = unsafe { &mut all_tasks };
        if handler.started {
            os_curr_task_id.borrow(cs_token).replace(curr);
        } else {
            handler.started = true;
        }
        handler.curr_tid = next;
        os_next_task_id.borrow(cs_token).replace(next);
        unsafe {
            cortex_m::peripheral::SCB::set_pendsv();
        }
    })
}

pub fn is_preemptive() -> bool {
    execute_critical(|_| unsafe { all_tasks.is_preemptive })
}

pub fn get_curr_tid() -> TaskId {
    execute_critical(|_| {
        let handler = unsafe { &mut all_tasks };
        return handler.curr_tid as TaskId;
    })
}

pub fn block_tasks(tasks_mask: BooleanVector) {
    execute_critical(|_| unsafe {
        all_tasks.block_tasks(tasks_mask);
    })
}

pub fn unblock_tasks(tasks_mask: BooleanVector) {
    execute_critical(|_| unsafe {
        all_tasks.unblock_tasks(tasks_mask);
    })
}

pub fn task_exit() {
    let curr_tid = get_curr_tid();
    execute_critical(|_| {
        unsafe { all_tasks.active_tasks &= !(1 << curr_tid as u32) };
    });
    schedule()
}

pub fn release(tasks_mask: BooleanVector) {
    execute_critical(|_| unsafe { all_tasks.release(tasks_mask) });
}

pub fn enable_preemption() {
    execute_critical(|_| unsafe { all_tasks.is_preemptive = true })
}

pub fn disable_preemption() {
    execute_critical(|_| unsafe {
        all_tasks.is_preemptive = false;
    })
}
