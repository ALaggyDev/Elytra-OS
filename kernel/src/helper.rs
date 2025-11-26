use core::arch::asm;

/// Halt and Catch Fire.
pub fn hcf() -> ! {
    loop {
        unsafe { asm!("hlt", options(nomem, nostack, preserves_flags)) };
    }
}
