use core::fmt;

use super::port::*;

const PORT: u16 = 0x3F8; // COM1

pub unsafe fn init() -> bool {
    unsafe {
        outb(PORT + 1, 0x00); // Disable all interrupts
        outb(PORT + 3, 0x80); // Enable DLAB (set baud rate divisor)
        outb(PORT, 0x03); // Set divisor to 3 (lo byte) 38400 baud
        outb(PORT + 1, 0x00); //                  (hi byte)
        outb(PORT + 3, 0x03); // 8 bits, no parity, one stop bit
        outb(PORT + 2, 0xC7); // Enable FIFO, clear them, with 14-byte threshold
        outb(PORT + 4, 0x0B); // IRQs enabled, RTS/DSR set
        outb(PORT + 4, 0x1E); // Set in loopback mode, test the serial chip
        outb(PORT, 0xAE); // Test serial chip (send byte 0xAE and check if serial returns same byte)

        // Check if serial is faulty (i.e: not same byte as sent)
        if inb(PORT) != 0xAE {
            return false;
        }

        // If serial is not faulty set it in normal operation mode
        // (not-loopback with IRQs enabled and OUT#1 and OUT#2 bits enabled)
        outb(PORT + 4, 0x0F);
        true
    }
}

pub fn received() -> bool {
    unsafe { (inb(PORT + 5) & 1) != 0 }
}

pub fn read() -> u8 {
    while !received() {}
    unsafe { inb(PORT) }
}

pub fn can_write() -> bool {
    unsafe { (inb(PORT + 5) & 0x20) != 0 }
}

pub fn write_u8(val: u8) {
    while !can_write() {}
    unsafe { outb(PORT, val) }
}

pub fn write_str(s: &str) {
    for byte in s.bytes() {
        if byte == b'\n' {
            write_u8(b'\r');
        }
        write_u8(byte);
    }
}

struct Serial;

static mut SERIAL: Serial = Serial;

impl fmt::Write for Serial {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        write_str(s);
        Ok(())
    }
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    unsafe {
        SERIAL.write_fmt(args).unwrap();
    }
}

#[macro_export]
macro_rules! printk {
    ($($arg:tt)*) => ($crate::io::serial::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! printkln {
    () => ($crate:printk!("\n"));
    ($($arg:tt)*) => ($crate::printk!("{}\n", format_args!($($arg)*)));
}
