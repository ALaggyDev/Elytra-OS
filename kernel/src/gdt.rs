use core::{arch::asm, mem};

use arbitrary_int::{u4, u20};
use bitbybit::bitfield;

#[bitfield(u64)]
struct Entry {
    #[bits([0..=15, 48..=51], rw)]
    limit: u20,

    #[bits([16..=39, 56..=63], rw)]
    base: u32,

    #[bits(40..=47, rw)]
    access: u8,

    #[bits(52..=55, rw)]
    flags: u4,
}

const SIZE_OF_GDT: usize = 5;

pub const KERNEL_CODE_SELECTOR: u16 = 0x08;
pub const KERNEL_DATA_SELECTOR: u16 = 0x10;
pub const USER_CODE_SELECTOR: u16 = 0x18 | 0x03;
pub const USER_DATA_SELECTOR: u16 = 0x20 | 0x03;

#[repr(C)]
struct Gdt([Entry; SIZE_OF_GDT]);

#[repr(C, packed)]
struct Gdtr {
    size: u16,
    base: *const Gdt,
}

static mut GDT: Gdt = Gdt([Entry::ZERO; SIZE_OF_GDT]);

static mut GDTR: Gdtr = Gdtr {
    size: 0,
    base: core::ptr::null(),
};

pub unsafe fn init() {
    // Setup gdt

    let gdt = unsafe { &mut GDT };
    // Null segment
    gdt.0[0] = Entry::ZERO;
    // Kernel code segment
    gdt.0[1] = Entry::ZERO
        .with_access(0b10011011)
        .with_flags(u4::new(0b0010));
    // Kernel data segment
    gdt.0[2] = Entry::ZERO
        .with_access(0b10010011)
        .with_flags(u4::new(0b0000));
    // User code segment
    gdt.0[3] = Entry::ZERO
        .with_access(0b11111011)
        .with_flags(u4::new(0b0010));
    // User data segment
    gdt.0[4] = Entry::ZERO
        .with_access(0b11110011)
        .with_flags(u4::new(0b0000));

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
