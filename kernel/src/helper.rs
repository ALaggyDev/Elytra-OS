use core::arch::asm;

use crate::consts;

/// Halt and Catch Fire.
pub fn hcf() -> ! {
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
