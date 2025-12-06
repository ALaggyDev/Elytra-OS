#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![allow(static_mut_refs)] // we allow references to static mut, because the kernel often uses global mutable state

extern crate alloc;

use bootloader_api::{BootInfo, BootloaderConfig, config::Mapping, entry_point};
use core::panic::PanicInfo;

use crate::consts::{KERNEL_OFFSET, PHYS_MEM_OFFSET};

pub mod consts;
pub mod gdt;
pub mod helper;
pub mod idt;
pub mod io;
pub mod isr;
pub mod mem;
pub mod primitives;
pub mod startup;
pub mod test;

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

entry_point!(kernel_entry, config = &BOOTLOADER_CONFIG);

pub static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::FixedAddress(PHYS_MEM_OFFSET as u64));
    config.mappings.kernel_base = Mapping::FixedAddress(KERNEL_OFFSET as u64);
    config.mappings.dynamic_range_start = Some(PHYS_MEM_OFFSET as u64);
    config
};
