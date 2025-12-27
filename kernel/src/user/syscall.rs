//! Syscalls using the SYSCALL/SYSRET instructions.
//!
//! Calling convention for userspace:
//!   RAX: syscall number
//!   RDI, RSI, RDX, R10, R8, R9: arguments
//!   RAX: return value
//!   Caller-saved and callee-saved registers are the same as System V AMD64 ABI.

use core::arch::naked_asm;

use crate::{
    msr::{IA32_EFER, IA32_FMASK, IA32_LSTAR, IA32_STAR, read_msr, write_msr},
    printkln,
};

pub fn init() {
    // Enable SYSCALL/SYSRET in IA32_EFER
    write_msr(IA32_EFER, read_msr(IA32_EFER) | 1);

    // Setup segment selectors in IA32_STAR
    //
    // For SYSCALL, we set IA32_STAR[47:32] to 0x8:
    //   CS: IA32_STAR[47:32]        = 0x8 (Kernel code segment)
    //   SS: IA32_STAR[47:32] + 8    = 0x10 (Kernel data segment)
    //
    // For SYSRET, we set IA32_STAR[63:48] to 0x10:
    //   CS: IA32_STAR[63:48] + 16   = 0x20 (User code segment)
    //   SS: IA32_STAR[63:48] + 8    = 0x18 (User data segment)
    write_msr(IA32_STAR, (0x8 << 32) | (0x10 << 48));

    // Set syscall entry address in IA32_LSTAR
    write_msr(IA32_LSTAR, syscall_entry as *const () as u64);

    // Set flags mask in IA32_FMASK (disable interrupt, clear direction flag)
    write_msr(IA32_FMASK, 0x300);
}

// Ideally, this should be stored in the per-cpu data structure referenced by GS base.
pub static mut USER_RSP: usize = 0;
pub static mut KERNEL_STACK_ADDR: usize = 0;

#[repr(C)]
#[derive(Debug)]
pub struct SyscallArgs {
    pub num: usize,  // rax
    pub arg1: usize, // rdi
    pub arg2: usize, // rsi
    pub arg3: usize, // rdx
    pub arg4: usize, // r10
    pub arg5: usize, // r8
    pub arg6: usize, // r9
}

#[unsafe(naked)]
pub extern "C" fn syscall_entry() {
    naked_asm!(
        "mov [rip + {0}], rsp",      // Save user rsp temporarily
        "mov rsp, [rip + {1}]",      // Load kernel stack rsp

        "push [rip + {0}]",          // Save user rsp

        "push r11",                  // Save r11 (user rflags)
        "push rcx",                  // Save rcx (user rip)

        "push r9",                   // Save syscall arguments in SyscallArgs struct
        "push r8",
        "push r10",
        "push rdx",
        "push rsi",
        "push rdi",
        "push rax",

        "mov rdi, rsp",              // First argument: pointer to SyscallArgs

        // syscall_handler follows System V, so caller-saved registers should be preserved.
        // So we don't have to save them manually here.
        // However, if the execution messes up, we might leak data to user mode or mess up user mode.
        // We might need to assess if such a risk is acceptable in the future.
        "call {2}",                  // Call syscall handler

        "add rsp, 56",               // Clean up SyscallArgs

        "xor rdi, rdi",              // Clear registers to prevent leaking data to user mode
        "xor rsi, rsi",              // (caller-saved registers, rax, rcx and r11 are ignored)
        "xor rdx, rdx",
        "xor r8, r8",
        "xor r9, r9",
        "xor r10, r10",

        "pop rcx",                   // Restore rcx (user rip)
        "pop r11",                   // Restore r11 (user rflags)

        "pop rsp",                   // Restore user rsp

        "sysretq",                   // Return to user mode

        sym USER_RSP,
        sym KERNEL_STACK_ADDR,
        sym syscall_handler
    )
}

pub extern "C" fn syscall_handler(args: &mut SyscallArgs) -> usize {
    printkln!("Syscall received! Args: {:#x?}", args);

    0
}
