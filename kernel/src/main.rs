#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![allow(static_mut_refs)] // we allow references to static mut, because the kernel often uses global mutable state

use bootloader_api::{BootInfo, entry_point};
use core::panic::PanicInfo;

mod gdt;
mod helper;
mod idt;
mod io;
mod isr;
mod startup;

/// This function is called on panic.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    printkln!("Kernel panic!\n{:?}", info);
    helper::hcf();
}

/// Entry point for the kernel.
fn kernel_entry(boot_info: &'static mut BootInfo) -> ! {
    startup::kernel_main(boot_info);
}

entry_point!(kernel_entry);
