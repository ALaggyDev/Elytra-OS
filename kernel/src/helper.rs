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

#[inline]
pub fn align_down(addr: usize, align: usize) -> usize {
    addr / align * align
}

#[inline]
pub fn align_up(addr: usize, align: usize) -> usize {
    align_down(addr + (align - 1), align)
}

#[inline]
pub fn log2_floor(x: usize) -> usize {
    x.ilog2() as usize
}

#[inline]
pub fn log2_ceil(x: usize) -> usize {
    log2_floor(x) + !x.is_power_of_two() as usize
}
