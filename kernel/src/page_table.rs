use core::arch::asm;

use crate::helper::{p2v, v2p};

#[repr(C, align(4096))]
pub struct PageTable(pub [u64; 512]);

/// Get the (virtual) address of the active level 4 page table.
pub unsafe fn get_active_page_table() -> *mut PageTable {
    let level_4_table: usize;
    unsafe {
        asm!("mov {}, cr3", out(reg) level_4_table, options(nomem, nostack, preserves_flags))
    };
    p2v(level_4_table) as *mut PageTable
}

/// Set the active level 4 page table (virtual address).
pub unsafe fn set_active_page_table(addr: *const PageTable) {
    let phys_addr = v2p(addr as usize);
    unsafe { asm!("mov cr3, {}", in(reg) phys_addr, options(nomem, nostack, preserves_flags)) };
}
