//! # Task Management module
//! Defines Kernel routines which will take care of Task management functionality.
//! Declares a global instance of Scheduler that will be used by the Kernel routines to provide the functionality.

use cortex_m::interrupt::free as execute_critical;
use cortex_m::interrupt::Mutex;
use core::cell::RefCell;

use crate::KernelError;
use crate::priv_execute;
use crate::system::task_manager::*;
use crate::system::types::{BooleanVector, TaskId};
use crate::utils::arch::{is_privileged, svc_call};

/// Global Scheduler instance
#[no_mangle]
pub static all_tasks: Mutex<RefCell<Scheduler>> = Mutex::new(RefCell::new(Scheduler::new()));

pub static mut os_curr_task_id: usize = 0;
pub static mut os_next_task_id: usize = 0;

/// Initializes the Kernel scheduler. `is_preemptive` defines if the Kernel should operating preemptively 
/// or not. This method sets the `is_preemptive` field of the Scheduler instance and creates the idle task. 
/// The idle task is created with zero priority; hence, it is only executed when no other task is in Ready state.
pub fn init(is_preemptive: bool) -> Result<(),KernelError>{
    execute_critical(|cs_token| all_tasks.borrow(cs_token).borrow_mut().init(is_preemptive) )
}

/// Starts the Kernel scheduler, which starts scheduling tasks and starts the SysTick timer using the
/// reference of the Peripherals instance and the `tick_interval`. `tick_interval` specifies the
/// frequency of the timer interrupt. The SysTick exception updates the kernel regarding the time
/// elapsed, which is used to dispatch events and schedule tasks.
pub fn start_kernel() -> ! {
    loop {
        preempt();
    }
}

/// Create a new task with the configuration set as arguments passed.
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
        execute_critical(|cs_token| unsafe {
            all_tasks.borrow(cs_token).borrow_mut().create_task(priority as usize, stack, handler_fn, param)
        })
    })
}

/// This function is called from both privileged and unprivileged context.
/// Hence if the function is called from privileged context, then `preempt()` is called.
/// Else, the `svc_call()` is executed, this function creates the SVC exception.
/// And the SVC handler calls schedule again. Thus, the permission level is raised to privileged via the exception.
pub fn schedule() {
    match is_privileged() {
        true => preempt(),
        false => svc_call(),
    };
}

/// If the scheduler is running and the highest priority task and currently running task aren’t the same, 
/// then the `context_switch()` is called.
pub fn preempt() {
    execute_critical(|cs_token| {
        let handler = &mut all_tasks.borrow(cs_token).borrow_mut();
        let next_tid = handler.get_next_tid();
        let curr_tid = handler.curr_tid as TaskId;
        if curr_tid != next_tid{
            unsafe {
                os_curr_task_id = curr_tid as usize;
                os_next_task_id = next_tid as usize;
                cortex_m::peripheral::SCB::set_pendsv();
                handler.curr_tid = os_next_task_id;
            }
        }
    })
}

/// Returns the TaskId of the currently running task in the kernel.
pub fn get_curr_tid() -> TaskId {
    execute_critical(|cs_token| {
        all_tasks.borrow(cs_token).borrow().curr_tid as TaskId
    })
}

/// The Kernel blocks the tasks mentioned in `tasks_mask`.
pub fn block_tasks(tasks_mask: BooleanVector) {
    execute_critical(|cs_token| all_tasks.borrow(cs_token).borrow_mut().block_tasks(tasks_mask))
}

/// The Kernel unblocks the tasks mentioned in tasks_mask.
pub fn unblock_tasks(tasks_mask: BooleanVector) {
    execute_critical(|cs_token| all_tasks.borrow(cs_token).borrow_mut().unblock_tasks(tasks_mask))
}

/// The `task_exit` function is called just after a task finishes execution. This function unsets the task’s corresponding bit in the `active_tasks` and calls schedule. Hence in the next call to schedule, the kernel schedules some other task.
pub fn task_exit() {
    let curr_tid = get_curr_tid();
    execute_critical(|cs_token| {
        let handler = &mut all_tasks.borrow(cs_token).borrow_mut();
        handler.active_tasks &= !(1 << curr_tid as u32);
    });
    schedule()
}

/// The Kernel releases the tasks in the `task_mask`, these tasks transition from the waiting to the ready state.
pub fn release(tasks_mask: BooleanVector) {
    execute_critical(|cs_token| all_tasks.borrow(cs_token).borrow_mut().release(tasks_mask));
}