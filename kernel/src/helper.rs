use core::arch::asm;

/// Halt and Catch Fire.
pub fn hcf() -> ! {
    loop {
        unsafe { asm!("hlt", options(nostack)) };
    }
}
