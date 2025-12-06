use alloc::{boxed::Box, vec};

use crate::{mem::buddy, printkln};

// Run test.
pub fn test() {
    printkln!("Here is a number: {}", 42);

    test_buddy_alloc();
    test_slab_alloc();
}

fn test_buddy_alloc() {
    unsafe {
        let ptr1 = buddy::alloc_pages_order(0);
        printkln!("Allocated page at: {:#x}", ptr1 as usize);

        let ptr2 = buddy::alloc_pages_order(1);
        printkln!("Allocated 2 pages at: {:#x}", ptr2 as usize);

        let ptr3 = buddy::alloc_pages_order(0);
        printkln!("Allocated 4 pages at: {:#x}", ptr3 as usize);

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
