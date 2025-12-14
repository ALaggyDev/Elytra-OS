use core::{arch::asm, mem::forget};

use crate::{
    consts::PAGE_SIZE,
    gdt::{TSS, USER_CODE_SELECTOR, USER_DATA_SELECTOR},
    isr::InterruptStackFrame,
    mem::buddy::alloc_pages_panic,
    user::{address_space::AddressSpace, syscall},
};

pub const USER_STACK_NUM_PAGES: usize = 2;
pub const KERNEL_STACK_NUM_PAGES: usize = 2;

pub fn test_task() {
    let user_code_vaddr = 0x400000;
    let user_stack_vaddr = 0x800000;

    let mut addr_space = AddressSpace::new();

    // Map kernel pages into the new address space
    addr_space.map_kernel_pages();

    // Create user code
    let user_code = addr_space
        .add_virt_region(user_code_vaddr, 0x1000, false, true)
        .unwrap();
    unsafe {
        // mov eax, 1
        // mov rdi, 2
        // mov rsi, 3
        // mov rdx, 4
        // mov r10, 5
        // mov r8, 6
        // mov r9, 7
        // syscall
        // hlt
        (user_code as *mut [u8; _]).write([
            0xB8, 0x01, 0x00, 0x00, 0x00, 0x48, 0xC7, 0xC7, 0x02, 0x00, 0x00, 0x00, 0x48, 0xC7,
            0xC6, 0x03, 0x00, 0x00, 0x00, 0x48, 0xC7, 0xC2, 0x04, 0x00, 0x00, 0x00, 0x49, 0xC7,
            0xC2, 0x05, 0x00, 0x00, 0x00, 0x49, 0xC7, 0xC0, 0x06, 0x00, 0x00, 0x00, 0x49, 0xC7,
            0xC1, 0x07, 0x00, 0x00, 0x00, 0x0F, 0x05, 0xF4,
        ]);
    }

    // Create user stack
    let _ = addr_space
        .add_virt_region(
            user_stack_vaddr,
            USER_STACK_NUM_PAGES * PAGE_SIZE,
            true,
            false,
        )
        .unwrap();

    // Create kernel stack
    let kernel_stack = unsafe { alloc_pages_panic(KERNEL_STACK_NUM_PAGES) };

    // Write the interrupt frame to the kernel stack
    let frame_ptr = unsafe {
        kernel_stack
            .add(KERNEL_STACK_NUM_PAGES * PAGE_SIZE)
            .sub(size_of::<InterruptStackFrame>())
    };
    unsafe {
        (frame_ptr as *mut InterruptStackFrame).write(InterruptStackFrame {
            ip: user_code_vaddr,
            cs: USER_CODE_SELECTOR as usize,
            flags: 0x202,
            sp: user_stack_vaddr + USER_STACK_NUM_PAGES * PAGE_SIZE,
            ss: USER_DATA_SELECTOR as usize,
        });
    }

    // Switch to the task
    unsafe {
        let kernel_stack_top = kernel_stack.add(KERNEL_STACK_NUM_PAGES * PAGE_SIZE);

        // Set tss rsp0
        TSS.rsp0 = kernel_stack_top as u64;

        // Set syscall stack pointer
        syscall::KERNEL_STACK_ADDR = kernel_stack_top as usize;

        // Switch address space
        addr_space.switch_to_this();
        forget(addr_space);

        asm!(
            "mov rsp, {}",

            "mov rax, 0",
            "mov rbx, 0",
            "mov rcx, 0",
            "mov rdx, 0",
            "mov rsi, 0",
            "mov rdi, 0",
            "mov rbp, 0",
            "mov r8, 0",
            "mov r9, 0",
            "mov r10, 0",
            "mov r11, 0",
            "mov r12, 0",
            "mov r13, 0",
            "mov r14, 0",
            "mov r15, 0",

            "iretq",
            in(reg) frame_ptr,
            options(nostack, noreturn)
        );
    }
}
