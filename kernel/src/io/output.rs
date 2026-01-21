use core::fmt::{self, Write};

use bootloader_api::BootInfo;
use spin::Mutex;

use crate::{
    idt::without_interrupt,
    io::{framebuffer::FrameBufferWriter, serial::Serial},
};

static SERIAL: Mutex<Option<Serial>> = Mutex::new(None);
static FRAMEBUFFER: Mutex<Option<FrameBufferWriter>> = Mutex::new(None);

pub fn init(boot_info: &mut BootInfo) {
    // Initialize serial port
    *SERIAL.lock() = Some(Serial::new().unwrap());

    // Initialize framebuffer writer, if available
    if let Some(framebuffer) = boot_info.framebuffer.take() {
        let info = framebuffer.info();
        let buffer = framebuffer.into_buffer();
        *FRAMEBUFFER.lock() = Some(FrameBufferWriter::new(buffer, info));
    }
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    without_interrupt(|| {
        // Print to serial, if available
        if let Some(serial) = SERIAL.lock().as_mut() {
            serial.write_fmt(args).unwrap();
        }

        // Print to framebuffer, if available
        if let Some(framebuffer) = FRAMEBUFFER.lock().as_mut() {
            framebuffer.write_fmt(args).unwrap();
        }
    });
}

#[macro_export]
macro_rules! printk {
    ($($arg:tt)*) => ($crate::io::output::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! printlnk {
    () => ($crate::printk!("\n"));
    ($($arg:tt)*) => ($crate::printk!("{}\n", format_args!($($arg)*)));
}
