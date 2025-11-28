use core::arch::asm;

use arbitrary_int::{u2, u3};
use bitbybit::{bitenum, bitfield};

use crate::{gdt::KERNEL_CODE_SELECTOR, isr};

#[bitenum(u4)]
enum GateType {
    InterruptGate = 0b1110,
    TrapGate = 0b1111,
}

#[bitfield(u128)]
struct Entry {
    #[bits([0..=15, 48..=95], rw)]
    offset: u64,

    #[bits(16..=31, rw)]
    selector: u16,

    #[bits(32..=34, rw)]
    ist: u3,

    #[bits(40..=43, w)]
    gate_type: GateType,

    #[bits(45..=46, rw)]
    dpl: u2,

    #[bit(47, rw)]
    present: bool,
}

#[repr(C)]
struct Idt([Entry; 256]);

#[repr(C, packed)]
struct Idtr {
    size: u16,
    base: *const Idt,
}

static mut IDT: Idt = Idt([Entry::ZERO; 256]);

static mut IDTR: Idtr = Idtr {
    size: 0,
    base: core::ptr::null(),
};

fn to_entry(func: *const ()) -> Entry {
    Entry::ZERO
        .with_offset(func as u64)
        .with_selector(KERNEL_CODE_SELECTOR)
        .with_ist(u3::new(0))
        .with_gate_type(GateType::InterruptGate)
        .with_dpl(u2::new(0))
        .with_present(true)
}

pub unsafe fn init() {
    // Setup idt

    let idt = unsafe { &mut IDT };
    idt.0[0] = to_entry(isr::isr_0 as *const ());
    idt.0[1] = to_entry(isr::isr_1 as *const ());
    idt.0[2] = to_entry(isr::isr_2 as *const ());
    idt.0[3] = to_entry(isr::isr_3 as *const ());
    idt.0[4] = to_entry(isr::isr_4 as *const ());
    idt.0[5] = to_entry(isr::isr_5 as *const ());
    idt.0[6] = to_entry(isr::isr_6 as *const ());
    idt.0[7] = to_entry(isr::isr_7 as *const ());
    idt.0[8] = to_entry(isr::isr_8 as *const ());
    idt.0[9] = to_entry(isr::isr_9 as *const ());
    idt.0[10] = to_entry(isr::isr_10 as *const ());
    idt.0[11] = to_entry(isr::isr_11 as *const ());
    idt.0[12] = to_entry(isr::isr_12 as *const ());
    idt.0[13] = to_entry(isr::isr_13 as *const ());
    idt.0[14] = to_entry(isr::isr_14 as *const ());
    idt.0[15] = to_entry(isr::isr_15 as *const ());
    idt.0[16] = to_entry(isr::isr_16 as *const ());
    idt.0[17] = to_entry(isr::isr_17 as *const ());
    idt.0[18] = to_entry(isr::isr_18 as *const ());
    idt.0[19] = to_entry(isr::isr_19 as *const ());
    idt.0[20] = to_entry(isr::isr_20 as *const ());
    idt.0[21] = to_entry(isr::isr_21 as *const ());

    // Setup idtr

    let idtr = unsafe { &mut IDTR };
    idtr.size = (core::mem::size_of::<Idt>() - 1) as u16;
    idtr.base = unsafe { &IDT } as *const Idt;

    // Load idt and set interrupt flag

    unsafe {
        asm!(
            "lidt [{}]",
            "sti",
            in(reg) idtr,
            options(nostack)
        );
    }
}
