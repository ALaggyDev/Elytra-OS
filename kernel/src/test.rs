use core::cell::UnsafeCell;

use alloc::{boxed::Box, rc::Rc, vec};

use crate::{
    consts::PAGE_SIZE,
    helper::p2v,
    mem::{
        buddy,
        page_table::{get_active_page_directory, resolve_virt_addr, set_active_page_directory},
    },
    printkln,
    user::{
        address_space::{AddressSpace, KERNEL_P4_TABLE},
        elf_parser::ElfParser,
        sched,
        task::Task,
    },
};

// Run test.
pub fn test() {
    printkln!("Here is a number: {}", 42);

    test_buddy_alloc();
    test_slab_alloc();
    test_paging();
    test_address_space();

    test_scheduler();
}

fn test_buddy_alloc() {
    unsafe {
        let ptr1 = buddy::alloc_pages_order(0);
        printkln!("Allocated 1 page at: {:#x}", ptr1 as usize);

        let ptr2 = buddy::alloc_pages_order(1);
        printkln!("Allocated 2 pages at: {:#x}", ptr2 as usize);

        let ptr3 = buddy::alloc_pages_order(0);
        printkln!("Allocated 1 page at: {:#x}", ptr3 as usize);

        ptr1.write_bytes(b'A', 64);

        printkln!(
            "Wrote to first allocated page: {:x?}",
            *(ptr1 as *const u64)
        );

        buddy::free_pages_order(ptr1, 0);
        buddy::free_pages_order(ptr2, 1);
        buddy::free_pages_order(ptr3, 0);
    }
}

fn test_slab_alloc() {
    let a = Box::new(2);
    let mut b = vec![1, 2, 3, 4, 5];

    printkln!("Boxed integer: {}", a);
    printkln!("Box address: {:#x}", &*a as *const i32 as usize);

    drop(a);

    let a = Box::new(3);

    printkln!("Boxed integer: {}", a);
    printkln!("Box address: {:#x}", &*a as *const i32 as usize);

    printkln!("Vector: {:?}", b);
    printkln!("Vector address: {:#x}", b.as_ptr() as usize);

    for i in 0..20 {
        b.push(i);
    }

    printkln!("Vector: {:?}", b);
    printkln!("Vector address: {:#x}", b.as_ptr() as usize);

    let c = Box::new([0u8; 6000]);
    printkln!(
        "Large Box address: {:#x}",
        &*c as *const [u8; 6000] as usize
    );

    drop(c);

    let c = Box::new([0u8; 6000]);
    printkln!(
        "Large Box address: {:#x}",
        &*c as *const [u8; 6000] as usize
    );
}

fn test_paging() {
    let val: usize = 0x1234_5678_9ABC_DEF0;

    unsafe {
        let phys_addr =
            resolve_virt_addr(get_active_page_directory(), &raw const val as usize).unwrap();
        printkln!("Physical address: {:#x}", phys_addr);

        let val_copy = *(p2v(phys_addr) as *const usize);
        printkln!("Value copied from physical address: {:#x}", val_copy);

        assert_eq!(val, val_copy);

        *(p2v(phys_addr) as *mut usize) = 0xdeadbeef;
        printkln!("Modified value: {:#x}", val);
    }
}

fn test_address_space() {
    let mut address_space = AddressSpace::new();

    address_space.map_kernel_pages();
    address_space
        .add_virt_region(0x400000, 4 * PAGE_SIZE, true, true)
        .unwrap();

    unsafe {
        set_active_page_directory(address_space.p4_table());

        *(0x403000 as *mut usize) = 0xCAFEBABE;

        set_active_page_directory(KERNEL_P4_TABLE);
    }
}

fn test_scheduler() {
    // From: https://users.rust-lang.org/t/can-i-conveniently-compile-bytes-into-a-rust-program-with-a-specific-alignment/24049/2
    #[repr(C)] // guarantee 'bytes' comes after '_align'
    struct AlignedAs<Align, Bytes: ?Sized> {
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

    const ELF_BINARY: &[u8] = include_bytes_align_as!(u64, "../../tests/test");

    let parser = ElfParser::parse(ELF_BINARY).unwrap();

    // Create tasks
    let task1 = Task::create_task_from_elf(&parser).unwrap();
    let task2 = Task::create_task_from_elf(&parser).unwrap();

    // Move tasks to heap
    let task1 = Rc::new(UnsafeCell::new(task1));
    let task2 = Rc::new(UnsafeCell::new(task2));

    unsafe {
        // Add tasks to scheduler
        sched::add_new_task(task1);
        sched::add_new_task(task2);

        // Begin scheduler (task1 should run first)
        sched::begin_scheduler();
    }
}
