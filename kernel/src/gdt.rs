use core::{arch::asm, mem};

const SIZE_OF_GDT: usize = 5;

#[derive(Debug)]
struct Gdt([u64; SIZE_OF_GDT]);

#[repr(C, packed)]
struct Gdtr {
    size: u16,
    base: *const Gdt,
}

static mut GDT: Gdt = Gdt([0; SIZE_OF_GDT]);

static mut GDTR: Gdtr = Gdtr {
    size: 0,
    base: core::ptr::null(),
};

pub const KERNEL_CODE_SELECTOR: u16 = 0x08;
pub const KERNEL_DATA_SELECTOR: u16 = 0x10;
pub const USER_CODE_SELECTOR: u16 = 0x18 | 0x03;
pub const USER_DATA_SELECTOR: u16 = 0x20 | 0x03;

fn segment_descriptor(base: u32, limit: u32, access: u8, flags: u8) -> u64 {
    let mut descriptor: u64 = 0;
    descriptor |= (limit as u64) & 0xFFFF; // Limit bits 0-15
    descriptor |= ((base as u64) & 0xFFFFFF) << 16; // Base bits 0-23
    descriptor |= (access as u64) << 40; // Access byte
    descriptor |= (((limit as u64) >> 16) & 0xF) << 48; // Limit bits 16-19
    descriptor |= ((flags as u64) & 0xF) << 52; // Flags
    descriptor |= (((base as u64) >> 24) & 0xFF) << 56; // Base bits 24-31
    return descriptor;
}

pub unsafe fn init() {
    // Setup gdt

    let gdt = unsafe { &mut GDT };
    // Null segment
    gdt.0[0] = 0;
    // Kernel code segment
    gdt.0[1] = segment_descriptor(0, 0, 0b10011011, 0b0010);
    // Kernel data segment
    gdt.0[2] = segment_descriptor(0, 0, 0b10010011, 0b0000);
    // User code segment
    gdt.0[3] = segment_descriptor(0, 0, 0b11111011, 0b0010);
    // User data segment
    gdt.0[4] = segment_descriptor(0, 0, 0b11110011, 0b0000);

    // Setup gdtr

    let gdtr = unsafe { &mut GDTR };
    gdtr.size = (mem::size_of::<Gdt>() - 1) as u16;
    gdtr.base = gdt as *const Gdt;

    // Load gdt
    unsafe {
        asm!(
            "lgdt [{0}]",
            in(reg) gdtr,
            options(nostack, preserves_flags),
        );
    }
}
