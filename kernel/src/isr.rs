use crate::{helper, printkln};

#[repr(C)]
#[derive(Debug)]
pub struct InterruptStackFrame {
    pub ip: usize,
    pub cs: usize,
    pub flags: usize,
    pub sp: usize,
    pub ss: usize,
}

const INTERRUPT_NAMES: [&'static str; 22] = [
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
