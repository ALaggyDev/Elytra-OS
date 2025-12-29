use core::{arch::naked_asm, cell::UnsafeCell, mem::offset_of, panic, ptr::null_mut};

use alloc::{collections::vec_deque::VecDeque, rc::Rc};

use crate::{
    consts,
    gdt::{TSS, Tss},
    user::{
        syscall,
        task::{KERNEL_STACK_SIZE, Task, TaskState},
    },
};

pub static mut CURRENT_TASK: Option<Rc<UnsafeCell<Task>>> = None;

pub static mut READY_TASKS: VecDeque<Rc<UnsafeCell<Task>>> = VecDeque::new();

// To use Rc<UnsafeCell<Task>> safely:
// We have to be very careful to not clone or drop any Rc ptr.
// Cloning Rc may prevent the task from being freed when it should be, and dropping Rc may free the task too early.

/// Begin the task scheduler. There must be at least one ready task in the ready queue.
pub unsafe fn begin_scheduler() -> ! {
    unsafe {
        let Some(next_task) = READY_TASKS.pop_front() else {
            panic!("No task to begin the scheduler!");
        };

        switch_task(next_task);
        panic!("begin_scheduler should never return!");
    }
}

/// Add a new task to the scheduler.
///
/// The task must be new, and this function must only be called once per task.
pub unsafe fn add_new_task(task: Rc<UnsafeCell<Task>>) {
    unsafe {
        READY_TASKS.push_back(task);
    }
}

/// Yield the current task.
/// If there is any ready task, this function will push the current task back to the ready queue and switch to another task.
/// Otherwise, continues the current task.
///
/// The following assumptions must hold:
/// 1. CURRENT_TASK must be Some.
/// 2. The current task is not in the terminated state.
pub unsafe fn yield_task() {
    unsafe {
        let Some(next_task) = READY_TASKS.pop_front() else {
            // No other ready task, continue the current task
            return;
        };

        switch_task(next_task);
    }
}

/// Switch to the given task.
/// This function will push the current task back to the ready queue and update CURRENT_TASK, then perform the context switch.
/// This function will return in the future when the task is switched back to this task.
///
/// The following assumptions must hold:
/// 1. CURRENT_TASK must be Some.
/// 2. Neither the current task nor the new task is in the terminated state.
pub unsafe fn switch_task(new_task: Rc<UnsafeCell<Task>>) {
    unsafe {
        let new_task_ptr = new_task.get();

        // Take the current task and replace it with the new task
        let old_task = CURRENT_TASK.replace(new_task);

        let old_task_ptr = old_task.as_ref().map_or(null_mut(), |v| v.get());

        // Put the current task back to the ready queue
        if let Some(old_task) = old_task {
            READY_TASKS.push_back(old_task);
        }

        // Perform the actual context switch
        inner_context_switch(old_task_ptr, new_task_ptr);
    }
}

/// The actual context switch.
/// This function will save the context of the old task and restore the context of the new task.
/// The caller must ensure that both tasks are not terminated (and not null).
///
/// Notably, this function does NOT update CURRENT_TASK or the ready queue. switch_task() is responsible for that.
///
/// new_task must not be null.
#[unsafe(naked)]
unsafe extern "C" fn inner_context_switch(old_task: *mut Task, new_task: *mut Task) {
    naked_asm!(
        // --- Old task ---

        // Check if old_task is null. If so, skip saving context.
        "test rdi, rdi",
        "jz .L_skip_old_task",

        // Save old task registers and rflags (only System V callee-saved registers)
        "pushfq",
        "push r15",
        "push r14",
        "push r13",
        "push r12",
        "push rbx",
        "push rbp",

        // Save old task kernel rsp
        "mov [rdi + {task_stack_krsp}], rsp",

        ".L_skip_old_task:",

        // --- New task ---

        // Set TSS rsp0 to the top of the kernel stack
        "mov rax, [rsi + {task_stack_ptr}]",
        "add rax, {kernel_stack_size}",
        "mov [rip + {tss} + {tss_rsp0}], rax",

        // Set syscall stack pointer
        "mov [rip + {syscall_stack_addr}], rax",

        // Switch page tables
        "mov rax, -{phys_mem_offset}",
        "add rax, [rsi + {task_page_table}]",
        "mov cr3, rax",

        // Switch kernel stack (essentially the crux of context switch)
        "mov rsp, [rsi + {task_stack_krsp}]",

        // Two cases:
        // 1. If the task is NEW, we need to clear all registers and return to user space using iretq.
        // 2. If the task is not NEW, we just switch the kernel stack.

        // Compare task.state with TaskState::New
        // If equal, jump to new task handling
        "cmp byte ptr [rsi + {task_state}], {new_state}",
        "je .L_new_task",

        // --- Existing task ---

        // Restore new task registers and rflags
        "pop rbp",
        "pop rbx",
        "pop r12",
        "pop r13",
        "pop r14",
        "pop r15",
        "popfq",

        // Return
        "ret",

        // --- New task ---

        ".L_new_task:",

        // Set task.state to Ready
        "mov byte ptr [rsi + {task_state}], {ready_state}",

        // Clear registers
        "xor rax, rax",
        "xor rbx, rbx",
        "xor rcx, rcx",
        "xor rdx, rdx",
        "xor rsi, rsi",
        "xor rdi, rdi",
        "xor rbp, rbp",
        "xor r8, r8",
        "xor r9, r9",
        "xor r10, r10",
        "xor r11, r11",
        "xor r12, r12",
        "xor r13, r13",
        "xor r14, r14",
        "xor r15, r15",

        // Return to user space
        "iretq",

        task_stack_ptr = const offset_of!(Task, kernel_stack.ptr),
        task_stack_krsp = const offset_of!(Task, kernel_stack.krsp),
        task_page_table = const offset_of!(Task, addr_space.p4_table),
        kernel_stack_size = const KERNEL_STACK_SIZE,

        tss = sym TSS,
        tss_rsp0 = const offset_of!(Tss, rsp0),

        phys_mem_offset = const consts::PHYS_MEM_OFFSET,

        syscall_stack_addr = sym syscall::KERNEL_STACK_ADDR,

        task_state = const offset_of!(Task, state),
        new_state = const TaskState::New as usize,
        ready_state = const TaskState::Ready as usize,
    )
}

// Old code:
// /// Switch to the given new task.
// pub fn switch_to_new_task(task: Task) -> ! {
//     unsafe {
//         // Set TSS rsp0 to the top of the kernel stack
//         TSS.rsp0 = task.kernel_stack.top() as u64;

//         // Set syscall stack pointer
//         syscall::KERNEL_STACK_ADDR = task.kernel_stack.top();

//         // Switch address space
//         task.addr_space.switch_to_this();

//         // Forget the task to avoid dropping it
//         let krsp = task.kernel_stack.krsp;
//         forget(task);

//         // Change kernel stack, clear registers and return to user space
//         asm!(
//             "mov rsp, {}",

//             "mov rax, 0",
//             "mov rbx, 0",
//             "mov rcx, 0",
//             "mov rdx, 0",
//             "mov rsi, 0",
//             "mov rdi, 0",
//             "mov rbp, 0",
//             "mov r8, 0",
//             "mov r9, 0",
//             "mov r10, 0",
//             "mov r11, 0",
//             "mov r12, 0",
//             "mov r13, 0",
//             "mov r14, 0",
//             "mov r15, 0",

//             "iretq",
//             in(reg) krsp,
//             options(nostack, noreturn)
//         );
//     }
// }
