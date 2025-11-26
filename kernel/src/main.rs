#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points

use bootloader_api::{BootInfo, entry_point};
use core::panic::PanicInfo;

mod helper;
mod io;
mod startup;

/// This function is called on panic.
#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    helper::hcf();
}

/// Entry point for the kernel.
fn kernel_entry(boot_info: &'static mut BootInfo) -> ! {
    startup::kernel_main(boot_info);
}

entry_point!(kernel_entry);
