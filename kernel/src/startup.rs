use core::ptr::slice_from_raw_parts_mut;

use bootloader_api::{BootInfo, info::MemoryRegionKind};

use crate::{
    gdt,
    helper::{self, p2v},
    idt::{self, enable_interrupt},
    io::serial,
    mem::{
        buddy,
        page_table::{self, PageDirectoryEntry},
    },
    printkln, test,
    user::{address_space::KERNEL_P4_TABLE, syscall},
};

pub(crate) fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    init(boot_info);

    test::test();

    helper::hcf();
}

// Initialize the kernel.
fn init(boot_info: &'static mut BootInfo) {
    unsafe {
        serial::init();
        init_mem_paging();
        gdt::init();
        idt::init();

        init_buddy_allocator(boot_info);

        syscall::init();

        enable_interrupt();
    }
}

// Set KERNEL_P4_TABLE and unmap all lower half memory.
// The rust-osdev bootloader leaves some junk in the lower half memory, so we have to unmap it ourselves.
// https://github.com/rust-osdev/bootloader/issues/470
fn init_mem_paging() {
    unsafe {
        let p4_table = page_table::get_active_page_directory();

        for i in 0..256 {
            // Each entry maps 512 GiB, so unmapping the first 256 entries unmaps the first 128 TiB.
            (*p4_table).0[i] = PageDirectoryEntry::ZERO;
        }

        // Flush the TLB by reloading CR3.
        page_table::set_active_page_directory(p4_table);

        KERNEL_P4_TABLE = p4_table;
    }
}

fn init_buddy_allocator(boot_info: &'static mut BootInfo) {
    let biggest_region = boot_info
        .memory_regions
        .iter()
        .filter(|region| region.kind == MemoryRegionKind::Usable)
        .max_by_key(|region| region.end - region.start)
        .unwrap();

    printkln!(
        "Initializing buddy allocator with region: {:#x} - {:#x}",
        biggest_region.start,
        biggest_region.end
    );

    unsafe {
        buddy::init(slice_from_raw_parts_mut(
            p2v(biggest_region.start as usize) as *mut u8,
            (biggest_region.end - biggest_region.start) as usize,
        ));
    }
}
