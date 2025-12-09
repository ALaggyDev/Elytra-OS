use core::arch::asm;

use arbitrary_int::{traits::Integer, u3, u4, u7, u9, u11, u12, u40};
use bitbybit::bitfield;

use crate::helper::{p2v, v2p};

// The official x86-64 names for these structures are complicated, so we use simpler names here.
// Top 3 page table levels: Page Directory (P4, P3, P2)
// Bottom page table level: Page Table (P1)

#[bitfield(u64)]
pub struct PageDirectoryEntry {
    #[bit(0, rw)]
    present: bool,
    #[bit(1, rw)]
    writable: bool,
    #[bit(2, rw)]
    user_accessible: bool,
    #[bit(3, rw)]
    write_through: bool,
    #[bit(4, rw)]
    cache_disable: bool,
    #[bit(5, rw)]
    accessed: bool,
    #[bit(6, rw)]
    available: bool,
    #[bit(7, rw)]
    page_size: bool,

    #[bits(8..=11, rw)]
    available_low: u4,

    #[bits(12..=51, rw)]
    inner_addr: u40,

    #[bits(52..=62, rw)]
    available_high: u11,

    #[bit(63, rw)]
    execute_disable: bool,
}

#[bitfield(u64)]
pub struct PageTableEntry {
    #[bit(0, rw)]
    present: bool,
    #[bit(1, rw)]
    writable: bool,
    #[bit(2, rw)]
    user_accessible: bool,
    #[bit(3, rw)]
    write_through: bool,
    #[bit(4, rw)]
    cache_disable: bool,
    #[bit(5, rw)]
    accessed: bool,
    #[bit(6, rw)]
    dirty: bool,
    #[bit(7, rw)]
    page_attribute_table: bool,
    #[bit(8, rw)]
    global: bool,

    #[bits(9..=11, rw)]
    available_low: u3,

    #[bits(12..=51, rw)]
    inner_addr: u40,

    #[bits(52..=58, rw)]
    available_high: u7,

    #[bits(59..=62, rw)]
    protection_key: u4,

    #[bit(63, rw)]
    execute_disable: bool,
}

impl PageDirectoryEntry {
    #[inline]
    pub const fn addr(&self) -> u64 {
        self.inner_addr().value() << 12
    }

    #[inline]
    pub fn with_addr(&self, addr: u64) -> Self {
        self.with_inner_addr(u40::masked_new(addr >> 12))
    }

    #[inline]
    pub fn set_addr(&mut self, addr: u64) {
        self.set_inner_addr(u40::masked_new(addr >> 12));
    }
}

impl PageTableEntry {
    #[inline]
    pub const fn addr(&self) -> u64 {
        self.inner_addr().value() << 12
    }

    #[inline]
    pub fn with_addr(&self, addr: u64) -> Self {
        self.with_inner_addr(u40::masked_new(addr >> 12))
    }

    #[inline]
    pub fn set_addr(&mut self, addr: u64) {
        self.set_inner_addr(u40::masked_new(addr >> 12));
    }
}

#[repr(C, align(4096))]
pub struct PageDirectory(pub [PageDirectoryEntry; 512]);

#[repr(C, align(4096))]
pub struct PageTable(pub [PageTableEntry; 512]);

#[bitfield(u64)]
pub struct VirtAddr {
    #[bits(0..=11, r)]
    offset: u12,
    #[bits(12..=20, r)]
    p1_index: u9,
    #[bits(21..=29, r)]
    p2_index: u9,
    #[bits(30..=38, r)]
    p3_index: u9,
    #[bits(39..=47, r)]
    p4_index: u9,
}

// Resolve a virtual address into a physical address given the P4 page directory.
// Page entry permissions are ignored.
pub unsafe fn resolve_virt_addr(p4_table: *mut PageDirectory, virt_addr: usize) -> Option<usize> {
    let virt_addr = VirtAddr::new_with_raw_value(virt_addr as u64);

    unsafe {
        let p4_entry = (*p4_table).0[virt_addr.p4_index().as_usize()];
        if !p4_entry.present() {
            return None;
        }

        let p3_table = p2v(p4_entry.addr() as usize) as *mut PageDirectory;
        let p3_entry = (*p3_table).0[virt_addr.p3_index().as_usize()];
        if !p3_entry.present() {
            return None;
        }

        let p2_table = p2v(p3_entry.addr() as usize) as *mut PageDirectory;
        let p2_entry = (*p2_table).0[virt_addr.p2_index().as_usize()];
        if !p2_entry.present() {
            return None;
        }

        let p1_table = p2v(p2_entry.addr() as usize) as *mut PageTable;
        let p1_entry = (*p1_table).0[virt_addr.p1_index().as_usize()];
        if !p1_entry.present() {
            return None;
        }

        let phys_addr = p1_entry.addr() as usize + virt_addr.offset().as_usize();
        Some(phys_addr)
    }
}

/// Get the (virtual) address of the active P4 page directory.
pub unsafe fn get_active_page_directory() -> *mut PageDirectory {
    let p4_table: usize;
    unsafe { asm!("mov {}, cr3", out(reg) p4_table, options(nomem, nostack, preserves_flags)) };
    p2v(p4_table) as *mut PageDirectory
}

/// Set the active P4 page directory (virtual address).
pub unsafe fn set_active_page_directory(addr: *const PageDirectory) {
    let phys_addr = v2p(addr as usize);
    unsafe { asm!("mov cr3, {}", in(reg) phys_addr, options(nomem, nostack, preserves_flags)) };
}
