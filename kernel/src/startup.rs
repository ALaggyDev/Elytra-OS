use bootloader_api::BootInfo;

use crate::{
    gdt, helper, idt,
    io::{port::outb, serial},
    printk, printkln,
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
        gdt::init();
        idt::init();
    }
}

// Disable the 8259 PIC.
fn pic_disable() {
    unsafe {
        outb(0x21, 0xFF);
        outb(0xA1, 0xFF);
    }
}

// Run test.
fn test() {
    printk!("printk works!\n");
    printkln!("Here is a number: {}", 42);
}
