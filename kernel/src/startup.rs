use core::ptr::slice_from_raw_parts_mut;

use bootloader_api::{BootInfo, info::MemoryRegionKind};

use crate::{
    gdt,
    helper::{self, p2v},
    idt,
    io::{port::outb, serial},
    mem::{buddy, page_table},
    printkln, test,
};

pub(crate) fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    init(boot_info);

    test::test();

    helper::hcf();
}

// Initialize the kernel.
fn init(boot_info: &'static mut BootInfo) {
    unsafe {
        pic_disable();
        serial::init();
        unmap_lower_half();
        gdt::init();
        idt::init();

        init_buddy_allocator(boot_info);
    }
}

// Disable the 8259 PIC. rust-osdev bootloader doesn't do this for us.
fn pic_disable() {
    unsafe {
        outb(0x21, 0xFF);
        outb(0xA1, 0xFF);
    }
}

// Unmap all lower half memory.
// The rust-osdev bootloader leaves some junk in the lower half memory, so we have to unmap it ourselves.
// https://github.com/rust-osdev/bootloader/issues/470
fn unmap_lower_half() {
    unsafe {
        let level_4_table = page_table::get_active_page_table();

        for i in 0..256 {
            // Each entry maps 512 GiB, so unmapping the first 256 entries unmaps the first 128 TiB.
            (*level_4_table).0[i] = 0;
        }

        // Flush the TLB by reloading CR3.
        page_table::set_active_page_table(level_4_table);
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
