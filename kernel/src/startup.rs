use bootloader_api::BootInfo;

use crate::{helper, io::serial, printk, printkln};

pub(crate) fn kernel_main(_: &'static mut BootInfo) -> ! {
    init();

    test();

    helper::hcf();
}

// Initialize the kernel.
fn init() {
    unsafe {
        serial::init();
    }
}

// Run test.
fn test() {
    printk!("printk works!\n");
    printkln!("Here is a number: {}", 42);

}
