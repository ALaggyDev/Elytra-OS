use core::fmt;

use super::port::*;

const PORT: u16 = 0x3F8; // COM1

pub struct Serial;

impl Serial {
    pub fn new() -> Result<Self, ()> {
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
                return Err(());
            }

            // If serial is not faulty set it in normal operation mode
            // (not-loopback with IRQs enabled and OUT#1 and OUT#2 bits enabled)
            outb(PORT + 4, 0x0F);
            Ok(Serial)
        }
    }

    pub fn can_read(&self) -> bool {
        unsafe { (inb(PORT + 5) & 1) != 0 }
    }

    pub fn read_u8(&self) -> u8 {
        while !self.can_read() {}
        unsafe { inb(PORT) }
    }

    pub fn can_write(&self) -> bool {
        unsafe { (inb(PORT + 5) & 0x20) != 0 }
    }

    pub fn write_u8(&self, val: u8) {
        while !self.can_write() {}
        unsafe { outb(PORT, val) }
    }
}

impl fmt::Write for Serial {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            if byte == b'\n' {
                self.write_u8(b'\r');
            }
            self.write_u8(byte);
        }
        Ok(())
    }
}
