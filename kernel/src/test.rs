use crate::{mem::buddy, printkln};

// Run test.
pub fn test() {
    printkln!("Here is a number: {}", 42);

    test_alloc();
}

fn test_alloc() {
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
