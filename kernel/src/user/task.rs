//! Task management.
//!
//! This module defines the `Task` struct, which represents a task (i.e. thread) in the OS.
//!
//! Kernel stack - New task, not executing:
//!
//! |---------------------| Low Address
//! |                     |
//! |        Empty        |
//! |                     |
//! |---------------------|
//! |                     | <- rsp
//! | x86 Interrupt frame |
//! |      for iretq      |
//! |---------------------| High Address
//!
//! Kernel stack - Normal task, context switched out, not executing:
//!
//! |---------------------| Low Address
//! |                     | <- rsp
//! |    Context switch   |
//! |      structure      |
//! |---------------------|
//! |                     |
//! |      Other data     |
//! |                     |
//! |---------------------|
//! |                     |
//! | x86 Interrupt frame |
//! |      for iretq      |
//! |---------------------| High Address

use alloc::boxed::Box;

use crate::{
    consts::PAGE_SIZE,
    gdt::{KERNEL_CODE_SELECTOR, KERNEL_DATA_SELECTOR, USER_CODE_SELECTOR, USER_DATA_SELECTOR},
    isr::InterruptStackFrame,
    mem::buddy::{alloc_pages_panic, free_pages},
    user::{address_space::AddressSpace, elf_parser::ElfParser},
};

pub const USER_STACK_SIZE: usize = 4 * PAGE_SIZE; // 16 KiB
pub const USER_STACK_VADDR: usize = 0x00007ffffff00000; // Bottom of user stack

pub const KERNEL_STACK_SIZE: usize = 4 * PAGE_SIZE; // 16 KiB

/// Represents a task (i.e. thread) in the OS.
#[derive(Debug)]
pub struct Task {
    pub state: TaskState,          // Current state of the task
    pub addr_space: AddressSpace,  // Address space of the task
    pub kernel_stack: KernelStack, // Kernel stack information
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    New,
    Ready,
    Terminated,
}

#[derive(Debug)]
pub struct KernelStack {
    pub ptr: *mut u8, // Pointer to the bottom of the kernel stack
    pub krsp: usize, // Kernel stack pointer. This is saved or resumed when the CPU is not executing this task.
}

impl KernelStack {
    pub fn new() -> Self {
        let ptr = unsafe { alloc_pages_panic(KERNEL_STACK_SIZE / PAGE_SIZE) };
        let krsp = ptr as usize + KERNEL_STACK_SIZE;

        KernelStack { ptr, krsp }
    }

    pub fn top(&self) -> usize {
        self.ptr as usize + KERNEL_STACK_SIZE
    }

    pub unsafe fn peek<T>(&self) -> *mut T {
        self.krsp as *mut T
    }

    pub unsafe fn push<T>(&mut self, value: T) {
        let size = size_of::<T>();
        self.krsp -= size;
        unsafe {
            let dst = self.krsp as *mut T;
            dst.write(value);
        }
    }

    pub unsafe fn pop<T>(&mut self) -> T {
        let size = size_of::<T>();
        let value = unsafe { (self.krsp as *mut T).read() };
        self.krsp += size;
        value
    }
}

impl Drop for KernelStack {
    fn drop(&mut self) {
        unsafe { free_pages(self.ptr, KERNEL_STACK_SIZE / PAGE_SIZE) };
    }
}

impl Task {
    pub fn create_user_task_from_elf(parser: &ElfParser) -> Result<Self, ()> {
        // Address space

        let mut addr_space = AddressSpace::new();

        // Map kernel pages into the new address space
        addr_space.map_kernel_pages();

        // Map ELF segments
        addr_space.map_elf_segments(parser)?;

        // Map user stack
        let _ = addr_space.add_virt_region(USER_STACK_VADDR, USER_STACK_SIZE, true, false)?;

        // Kernel stack

        let mut kernel_stack = KernelStack::new();
        unsafe {
            kernel_stack.push(InterruptStackFrame {
                ip: parser.get_header().e_entry as usize,
                cs: USER_CODE_SELECTOR as usize,
                flags: 0x202,
                sp: USER_STACK_VADDR + USER_STACK_SIZE,
                ss: USER_DATA_SELECTOR as usize,
            });
        }

        Ok(Task {
            state: TaskState::New,
            addr_space,
            kernel_stack,
        })
    }

    /// Create a kernel task with an argument passed to the entry point function.
    /// This function should be used in conjunction with the `kernel_task_trampoline!` macro.
    /// Real signature of entry point: `extern "C" fn(Box<T>) -> !`.
    pub fn create_kernel_task<T: Send>(entry: unsafe extern "C" fn() -> !, arg: Box<T>) -> Self {
        fn inner(entry: unsafe extern "C" fn() -> !, arg: *mut ()) -> Task {
            // Address space
            let mut addr_space = AddressSpace::new();

            // Map kernel pages into the new address space
            addr_space.map_kernel_pages();

            // Kernel stack
            let mut kernel_stack = KernelStack::new();
            unsafe {
                // Push argument
                kernel_stack.push(arg);

                // Push InterruptStackFrame
                kernel_stack.push(InterruptStackFrame {
                    ip: entry as usize,
                    cs: KERNEL_CODE_SELECTOR as usize,
                    flags: 0x202,
                    sp: kernel_stack.krsp,
                    ss: KERNEL_DATA_SELECTOR as usize,
                });
            }

            Task {
                state: TaskState::New,
                addr_space,
                kernel_stack,
            }
        }

        inner(entry, Box::into_raw(arg) as *mut ())
    }
}

#[macro_export]
macro_rules! kernel_task_trampoline {
    ($f:path) => {
        {
            #[unsafe(naked)]
            unsafe extern "C" fn trampoline() -> ! {
                core::arch::naked_asm!(
                    "mov rdi, [rsp]",
                    "jmp {func}",
                    func = sym $f,
                )
            }

            trampoline
        }
    };
}
