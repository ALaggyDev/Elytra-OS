use core::arch::asm;

use crate::{consts, printkln};

/// Halt and Catch Fire.
pub fn hcf() -> ! {
    printkln!("Halting CPU...");
    loop {
        unsafe { asm!("hlt", options(nomem, nostack, preserves_flags)) };
    }
}

/// Convert a physical address to a virtual address (in the direct mapping).
pub fn p2v(addr: usize) -> usize {
    addr + consts::PHYS_MEM_OFFSET
}

/// Convert a virtual address (in the direct mapping) to a physical address.
pub fn v2p(addr: usize) -> usize {
    addr - consts::PHYS_MEM_OFFSET
}
