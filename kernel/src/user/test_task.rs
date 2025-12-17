use core::{arch::asm, mem::forget, ptr::copy_nonoverlapping};

use crate::{
    consts::PAGE_SIZE,
    gdt::{TSS, USER_CODE_SELECTOR, USER_DATA_SELECTOR},
    helper::add_within_bounds,
    isr::InterruptStackFrame,
    mem::buddy::alloc_pages_panic,
    user::{
        address_space::AddressSpace, elf_parser::ElfParser, elf_structure::ElfProgramHeaderType,
        syscall,
    },
};

pub const USER_STACK_NUM_PAGES: usize = 2;
pub const KERNEL_STACK_NUM_PAGES: usize = 2;

pub fn test_task() {
    let user_code_vaddr = 0x400000;
    let user_stack_vaddr = 0x800000;

    let mut addr_space = AddressSpace::new();

    // Map kernel pages into the new address space
    addr_space.map_kernel_pages();

    // Create user code
    let user_code = addr_space
        .add_virt_region(user_code_vaddr, 0x1000, false, true)
        .unwrap();
    unsafe {
        // mov eax, 1
        // mov rdi, 2
        // mov rsi, 3
        // mov rdx, 4
        // mov r10, 5
        // mov r8, 6
        // mov r9, 7
        // syscall
        // hlt
        (user_code as *mut [u8; _]).write([
            0xB8, 0x01, 0x00, 0x00, 0x00, 0x48, 0xC7, 0xC7, 0x02, 0x00, 0x00, 0x00, 0x48, 0xC7,
            0xC6, 0x03, 0x00, 0x00, 0x00, 0x48, 0xC7, 0xC2, 0x04, 0x00, 0x00, 0x00, 0x49, 0xC7,
            0xC2, 0x05, 0x00, 0x00, 0x00, 0x49, 0xC7, 0xC0, 0x06, 0x00, 0x00, 0x00, 0x49, 0xC7,
            0xC1, 0x07, 0x00, 0x00, 0x00, 0x0F, 0x05, 0xF4,
        ]);
    }

    // Create user stack
    let _ = addr_space
        .add_virt_region(
            user_stack_vaddr,
            USER_STACK_NUM_PAGES * PAGE_SIZE,
            true,
            false,
        )
        .unwrap();

    // Create kernel stack
    let kernel_stack = unsafe { alloc_pages_panic(KERNEL_STACK_NUM_PAGES) };

    // Write the interrupt frame to the kernel stack
    let frame_ptr = unsafe {
        kernel_stack
            .add(KERNEL_STACK_NUM_PAGES * PAGE_SIZE)
            .sub(size_of::<InterruptStackFrame>())
    };
    unsafe {
        (frame_ptr as *mut InterruptStackFrame).write(InterruptStackFrame {
            ip: user_code_vaddr,
            cs: USER_CODE_SELECTOR as usize,
            flags: 0x202,
            sp: user_stack_vaddr + USER_STACK_NUM_PAGES * PAGE_SIZE,
            ss: USER_DATA_SELECTOR as usize,
        });
    }

    // Switch to the task
    unsafe {
        let kernel_stack_top = kernel_stack.add(KERNEL_STACK_NUM_PAGES * PAGE_SIZE);

        // Set tss rsp0
        TSS.rsp0 = kernel_stack_top as u64;

        // Set syscall stack pointer
        syscall::KERNEL_STACK_ADDR = kernel_stack_top as usize;

        // Switch address space
        addr_space.switch_to_this();
        forget(addr_space);

        asm!(
            "mov rsp, {}",

            "mov rax, 0",
            "mov rbx, 0",
            "mov rcx, 0",
            "mov rdx, 0",
            "mov rsi, 0",
            "mov rdi, 0",
            "mov rbp, 0",
            "mov r8, 0",
            "mov r9, 0",
            "mov r10, 0",
            "mov r11, 0",
            "mov r12, 0",
            "mov r13, 0",
            "mov r14, 0",
            "mov r15, 0",

            "iretq",
            in(reg) frame_ptr,
            options(nostack, noreturn)
        );
    }
}

pub fn load_elf(parser: &ElfParser) -> Result<AddressSpace, ()> {
    let mut addr_space = AddressSpace::new();

    // Map kernel pages into the new address space
    addr_space.map_kernel_pages();

    for i in 0..parser.get_header().e_phnum as usize {
        let ph = parser.get_program_header(i)?;

        if ph.p_type != ElfProgramHeaderType::Load {
            continue;
        }

        let mem_size = ph.p_memsz as usize;
        let file_size = ph.p_filesz as usize;
        let vaddr = ph.p_vaddr as usize;
        let offset = ph.p_offset as usize;

        let writable = (ph.p_flags & 0x2) != 0;
        let executable = (ph.p_flags & 0x1) != 0;

        // We check safety first
        add_within_bounds(offset, file_size, parser.get_buf().len()).ok_or(())?;
        if file_size > mem_size {
            return Err(());
        }

        // Create the virtual region
        let region = addr_space.add_virt_region(vaddr, mem_size, writable, executable)?;

        unsafe {
            // Copy the segment from the ELF file to memory
            // Additional memory are already zeroed by add_virt_region
            // Safety is checked above, so this *should* be safe
            copy_nonoverlapping(
                parser.get_buf().as_ptr().add(offset),
                region as *mut u8,
                file_size,
            );
        }
    }

    Ok(addr_space)
}

// From: https://users.rust-lang.org/t/can-i-conveniently-compile-bytes-into-a-rust-program-with-a-specific-alignment/24049/2
#[repr(C)] // guarantee 'bytes' comes after '_align'
pub struct AlignedAs<Align, Bytes: ?Sized> {
    pub _align: [Align; 0],
    pub bytes: Bytes,
}

macro_rules! include_bytes_align_as {
    ($align_ty:ty, $path:literal) => {{
        // const block expression to encapsulate the static
        use AlignedAs;

        // this assignment is made possible by CoerceUnsized
        static ALIGNED: &AlignedAs<$align_ty, [u8]> = &AlignedAs {
            _align: [],
            bytes: *include_bytes!($path),
        };

        &ALIGNED.bytes
    }};
}

const ELF_BINARY: &[u8] = include_bytes_align_as!(u64, "../../../tests/test");

pub fn test_load_elf() {
    let parser = ElfParser::parse(ELF_BINARY).unwrap();
    let mut addr_space = load_elf(&parser).unwrap();

    let user_stack_vaddr = 0x800000;

    // Create user stack
    let _ = addr_space
        .add_virt_region(
            user_stack_vaddr,
            USER_STACK_NUM_PAGES * PAGE_SIZE,
            true,
            false,
        )
        .unwrap();

    // Create kernel stack
    let kernel_stack = unsafe { alloc_pages_panic(KERNEL_STACK_NUM_PAGES) };

    // Write the interrupt frame to the kernel stack
    let frame_ptr = unsafe {
        kernel_stack
            .add(KERNEL_STACK_NUM_PAGES * PAGE_SIZE)
            .sub(size_of::<InterruptStackFrame>())
    };
    unsafe {
        (frame_ptr as *mut InterruptStackFrame).write(InterruptStackFrame {
            ip: parser.get_header().e_entry as usize,
            cs: USER_CODE_SELECTOR as usize,
            flags: 0x202,
            sp: user_stack_vaddr + USER_STACK_NUM_PAGES * PAGE_SIZE,
            ss: USER_DATA_SELECTOR as usize,
        });
    }

    // Switch to the task
    unsafe {
        let kernel_stack_top = kernel_stack.add(KERNEL_STACK_NUM_PAGES * PAGE_SIZE);

        // Set tss rsp0
        TSS.rsp0 = kernel_stack_top as u64;

        // Set syscall stack pointer
        syscall::KERNEL_STACK_ADDR = kernel_stack_top as usize;

        // Switch address space
        addr_space.switch_to_this();
        forget(addr_space);

        asm!(
            "mov rsp, {}",

            "mov rax, 0",
            "mov rbx, 0",
            "mov rcx, 0",
            "mov rdx, 0",
            "mov rsi, 0",
            "mov rdi, 0",
            "mov rbp, 0",
            "mov r8, 0",
            "mov r9, 0",
            "mov r10, 0",
            "mov r11, 0",
            "mov r12, 0",
            "mov r13, 0",
            "mov r14, 0",
            "mov r15, 0",

            "iretq",
            in(reg) frame_ptr,
            options(nostack, noreturn)
        );
    }
}
