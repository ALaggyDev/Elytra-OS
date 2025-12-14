use core::{arch::asm, mem::MaybeUninit};

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

// We will define 7 entries in the GDT:
// 0: Null segment
// 1: Kernel code segment
// 2: Kernel data segment
// 3: User data segment
// 4: User code segment
// 5: Task State Segment (lower half)
// 6: Task State Segment (higher half)

const SIZE_OF_GDT: usize = 7;

pub const KERNEL_CODE_SELECTOR: u16 = 0x08;
pub const KERNEL_DATA_SELECTOR: u16 = 0x10;
pub const USER_DATA_SELECTOR: u16 = 0x18 | 0x03;
pub const USER_CODE_SELECTOR: u16 = 0x20 | 0x03;

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

// TSS

#[repr(C, packed)]
pub struct Tss {
    _reserved0: u32,
    pub rsp0: u64,
    pub rsp1: u64,
    pub rsp2: u64,
    _reserved1: u64,
    pub ist1: u64,
    pub ist2: u64,
    pub ist3: u64,
    pub ist4: u64,
    pub ist5: u64,
    pub ist6: u64,
    pub ist7: u64,
    _reserved2: u64,
    _reserved3: u16,
    pub io_map_base: u16,
}

pub static mut TSS: Tss = unsafe { MaybeUninit::zeroed().assume_init() };

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
    // User data segment
    gdt.0[3] = Entry::ZERO
        .with_access(0b11110011)
        .with_flags(u4::new(0b0000));
    // User code segment
    gdt.0[4] = Entry::ZERO
        .with_access(0b11111011)
        .with_flags(u4::new(0b0010));
    // Task State Segment
    gdt.0[5] = Entry::ZERO
        .with_base(&raw const TSS as u32)
        .with_limit(u20::new(size_of::<Tss>() as u32 - 1))
        .with_access(0b10001001)
        .with_flags(u4::new(0b0000));
    gdt.0[6] = Entry::new_with_raw_value(&raw const TSS as u64 >> 32);

    // Setup gdtr

    let gdtr = unsafe { &mut GDTR };
    gdtr.size = (size_of::<Gdt>() - 1) as u16;
    gdtr.base = gdt as *const Gdt;

    unsafe {
        // Load gdt
        asm!(
            "lgdt [{0}]",
            in(reg) gdtr,
            options(nostack, preserves_flags),
        );

        // Load tss
        asm!("mov ax, 0x28", "ltr ax", out("ax") _, options(nostack, preserves_flags));
    }
}
