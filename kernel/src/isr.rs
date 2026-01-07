use pc_keyboard::{DecodedKey, HandleControl, Keyboard, ScancodeSet1, layouts::Us104Key};

use crate::{helper, idt::PICS, io::port::inb, printk, printkln};

// Interrupts are enabled for most of the time in the kernel.
// For code that should not be interrupted (e.g. context switch), use cli/sti instructions.
//
// Basic rules for interrupt handlers: (Interrupt 101)
// 1. Should be short (this improves input latency and avoid making mistakes)
// 2. Should be "async-safe" (no locks, no allocations, etc.)
// 3. Should prevent re-entrancy (to avoid stack overflow) (e.g. using interrupt gate, avoid nested interrupt, etc.)
// Every function that interrupt handlers call should also follow these rules.

#[repr(C)]
#[derive(Debug)]
pub struct InterruptStackFrame {
    pub ip: usize,
    pub cs: usize,
    pub flags: usize,
    pub sp: usize,
    pub ss: usize,
}

impl InterruptStackFrame {
    /// Returns true if the interrupt occurred in user mode.
    pub fn is_user_mode(&self) -> bool {
        self.cs & 0b11 != 0
    }
}

const INTERRUPT_NAMES: [&str; 22] = [
    "Division Error",
    "Debug Exception",
    "NMI Interrupt",
    "Breakpoint",
    "Overflow",
    "BOUND Range Exceeded",
    "Invalid Opcode",
    "Device Not Available",
    "Double Fault",
    "Coprocessor Segment Overrun",
    "Invalid TSS",
    "Segment Not Present",
    "Stack-Segment Fault",
    "General Protection",
    "Page Fault",
    "Reserved",
    "x87 FPU Floating-Point Error",
    "Alignment Check",
    "Machine Check",
    "SIMD Floating-Point Exception",
    "Virtualization Exception",
    "Control Protection Exception",
];

fn print_info(num: usize, frame: &InterruptStackFrame) {
    printkln!(
        "Received interrupt: {}\nFrame: {:#x?}",
        INTERRUPT_NAMES[num],
        frame
    );
}

fn print_info_with_err(num: usize, frame: &InterruptStackFrame, err_code: usize) {
    printkln!(
        "Received interrupt: {}\nFrame: {:#x?}\nError Code: {:#x}",
        INTERRUPT_NAMES[num],
        frame,
        err_code
    );
}

pub(super) unsafe extern "x86-interrupt" fn isr_0(frame: InterruptStackFrame) {
    print_info(0, &frame);
    helper::hcf();
}

pub(super) unsafe extern "x86-interrupt" fn isr_1(frame: InterruptStackFrame) {
    print_info(1, &frame);
    helper::hcf();
}

pub(super) unsafe extern "x86-interrupt" fn isr_2(frame: InterruptStackFrame) {
    print_info(2, &frame);
    helper::hcf();
}

pub(super) unsafe extern "x86-interrupt" fn isr_3(frame: InterruptStackFrame) {
    print_info(3, &frame);
    helper::hcf();
}

pub(super) unsafe extern "x86-interrupt" fn isr_4(frame: InterruptStackFrame) {
    print_info(4, &frame);
    helper::hcf();
}

pub(super) unsafe extern "x86-interrupt" fn isr_5(frame: InterruptStackFrame) {
    print_info(5, &frame);
    helper::hcf();
}

pub(super) unsafe extern "x86-interrupt" fn isr_6(frame: InterruptStackFrame) {
    print_info(6, &frame);
    helper::hcf();
}

pub(super) unsafe extern "x86-interrupt" fn isr_7(frame: InterruptStackFrame) {
    print_info(7, &frame);
    helper::hcf();
}

pub(super) unsafe extern "x86-interrupt" fn isr_8(frame: InterruptStackFrame, err_code: usize) {
    print_info_with_err(8, &frame, err_code);
    helper::hcf();
}

pub(super) unsafe extern "x86-interrupt" fn isr_9(frame: InterruptStackFrame) {
    print_info(9, &frame);
    helper::hcf();
}

pub(super) unsafe extern "x86-interrupt" fn isr_10(frame: InterruptStackFrame, err_code: usize) {
    print_info_with_err(10, &frame, err_code);
    helper::hcf();
}

pub(super) unsafe extern "x86-interrupt" fn isr_11(frame: InterruptStackFrame, err_code: usize) {
    print_info_with_err(11, &frame, err_code);
    helper::hcf();
}

pub(super) unsafe extern "x86-interrupt" fn isr_12(frame: InterruptStackFrame, err_code: usize) {
    print_info_with_err(12, &frame, err_code);
    helper::hcf();
}

pub(super) unsafe extern "x86-interrupt" fn isr_13(frame: InterruptStackFrame, err_code: usize) {
    print_info_with_err(13, &frame, err_code);
    helper::hcf();
}

pub(super) unsafe extern "x86-interrupt" fn isr_14(frame: InterruptStackFrame, err_code: usize) {
    print_info_with_err(14, &frame, err_code);
    helper::hcf();
}

pub(super) unsafe extern "x86-interrupt" fn isr_15(frame: InterruptStackFrame) {
    print_info(15, &frame);
    helper::hcf();
}

pub(super) unsafe extern "x86-interrupt" fn isr_16(frame: InterruptStackFrame) {
    print_info(16, &frame);
    helper::hcf();
}

pub(super) unsafe extern "x86-interrupt" fn isr_17(frame: InterruptStackFrame, err_code: usize) {
    print_info_with_err(17, &frame, err_code);
    helper::hcf();
}

pub(super) unsafe extern "x86-interrupt" fn isr_18(frame: InterruptStackFrame) {
    print_info(18, &frame);
    helper::hcf();
}

pub(super) unsafe extern "x86-interrupt" fn isr_19(frame: InterruptStackFrame) {
    print_info(19, &frame);
    helper::hcf();
}

pub(super) unsafe extern "x86-interrupt" fn isr_20(frame: InterruptStackFrame) {
    print_info(20, &frame);
    helper::hcf();
}

pub(super) unsafe extern "x86-interrupt" fn isr_21(frame: InterruptStackFrame, err_code: usize) {
    print_info_with_err(21, &frame, err_code);
    helper::hcf();
}

// --- Interrupt by PICs ---

// Vector: 0x20
pub(super) unsafe extern "x86-interrupt" fn pic_timer_handler(_: InterruptStackFrame) {
    printk!(".");

    unsafe { PICS.notify_end_of_interrupt(0x20) };
}

static mut KEYBOARD: Keyboard<Us104Key, ScancodeSet1> =
    Keyboard::new(ScancodeSet1::new(), Us104Key, HandleControl::Ignore);

// Vector: 0x21
pub(super) unsafe extern "x86-interrupt" fn pic_keyboard_handler(_: InterruptStackFrame) {
    let scancode = unsafe { inb(0x60) };

    let keyboard = unsafe { &mut KEYBOARD };
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::RawKey(key) => {
                    printk!("{:?}", key);
                }
                DecodedKey::Unicode(character) => {
                    printk!("{}", character);
                }
            }
        }
    }

    unsafe { PICS.notify_end_of_interrupt(0x21) };
}
