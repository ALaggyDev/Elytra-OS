use bootloader_api::BootInfo;

use crate::{
    gdt, helper, idt,
    io::{port::outb, serial},
    page_table, printk, printkln,
};

pub(crate) fn kernel_main(_: &'static mut BootInfo) -> ! {
    init();

    test();

    helper::hcf();
}

// Initialize the kernel.
fn init() {
    unsafe {
        pic_disable();
        serial::init();
        unmap_lower_half();
        gdt::init();
        idt::init();
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

// Run test.
fn test() {
    printk!("printk works!\n");
    printkln!("Here is a number: {}", 42);
}
