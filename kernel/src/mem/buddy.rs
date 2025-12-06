use core::{mem::MaybeUninit, ptr, slice};

use bitvec::slice::BitSlice;

use crate::{
    consts::PAGE_SIZE,
    helper::{align_up, log2_ceil},
    primitives::DoublyListHead,
};

pub const MAX_ORDER: usize = 10;

pub const SIZE_OF_MAX_ORDER: usize = PAGE_SIZE << MAX_ORDER;

struct Bucket {
    free_list: DoublyListHead,
    bitmap: *mut BitSlice<u8>,
}

pub struct BuddyAllocator {
    memory: *mut [u8],
    used: usize,
    buckets: [Bucket; MAX_ORDER + 1],
}

pub static mut BUDDY_ALLOCATOR: BuddyAllocator = unsafe { MaybeUninit::zeroed().assume_init() };

pub unsafe fn init(memory: *mut [u8]) {
    let allocator = unsafe { &mut BUDDY_ALLOCATOR };

    // Initialize free lists and bitmaps for each order.

    let mut cur_num = memory.len() / SIZE_OF_MAX_ORDER;
    let mut cur_ptr = memory as *mut u8;
    for order in (0..=MAX_ORDER).rev() {
        let bitmap_size = cur_num.div_ceil(8);
        let bitmap = BitSlice::<u8>::from_slice_mut(unsafe {
            slice::from_raw_parts_mut(cur_ptr, bitmap_size)
        });

        unsafe { DoublyListHead::new_empty(&raw mut allocator.buckets[order].free_list) };
        allocator.buckets[order].bitmap = bitmap as *mut _;

        cur_num *= 2;
        cur_ptr = unsafe { cur_ptr.add(bitmap_size) };
    }

    // Align the remaining memory to SIZE_OF_MAX_ORDER.

    let final_ptr = align_up(cur_ptr as usize, SIZE_OF_MAX_ORDER) as *mut u8;
    let final_len = memory.addr() + memory.len() - final_ptr as usize;

    allocator.memory = ptr::slice_from_raw_parts_mut(final_ptr, final_len);
    allocator.used = 0;
}

// Get bit index for bitmap.
fn bit_idx(page: *mut u8, order: usize) -> usize {
    let offset = page as usize - unsafe { &BUDDY_ALLOCATOR }.memory.addr();
    (offset / PAGE_SIZE) >> (order + 1)
}

// Toggle bitmap at index.
fn toggle_bitmap(bitmap: &mut BitSlice<u8>, idx: usize) {
    bitmap.set(idx, !bitmap[idx]);
}

pub unsafe fn alloc_pages_order(order: usize) -> *mut u8 {
    assert!(order <= MAX_ORDER);

    let allocator = unsafe { &mut BUDDY_ALLOCATOR };
    let bucket = &mut allocator.buckets[order];

    // Search for a free block in the free list
    if unsafe { !DoublyListHead::is_empty(&raw mut bucket.free_list) } {
        // Found a free block
        let page = bucket.free_list.next;

        // Remove the block from the free list
        unsafe { DoublyListHead::delete(page) };

        // Toggle the bitmap
        if order != MAX_ORDER {
            toggle_bitmap(
                unsafe { &mut *bucket.bitmap },
                bit_idx(page as *mut u8, order),
            );
        }

        page as *mut u8
    } else if order == MAX_ORDER {
        // Allocate a new block from the memory pool

        if allocator.used + SIZE_OF_MAX_ORDER > allocator.memory.len() {
            return ptr::null_mut();
        }

        let page = unsafe { (allocator.memory as *mut u8).add(allocator.used) };
        allocator.used += SIZE_OF_MAX_ORDER;

        page
    } else {
        // Otherwise, try to split a larger block
        let buddies = unsafe { alloc_pages_order(order + 1) };
        if buddies.is_null() {
            return ptr::null_mut();
        }

        // Calculate the address of the buddy block
        let buddy = unsafe { buddies.add(PAGE_SIZE << order) };

        // Insert the buddy into the free list
        unsafe {
            DoublyListHead::insert_after(&raw mut bucket.free_list, buddy as *mut DoublyListHead)
        };

        // Toggle the bitmap
        toggle_bitmap(unsafe { &mut *bucket.bitmap }, bit_idx(buddy, order));

        buddies
    }
}

pub unsafe fn free_pages_order(page: *mut u8, order: usize) {
    assert!(order <= MAX_ORDER);

    let allocator = unsafe { &mut BUDDY_ALLOCATOR };
    let bucket = &mut allocator.buckets[order];

    if order == MAX_ORDER {
        // Just add it to the free list
        unsafe {
            DoublyListHead::insert_after(&raw mut bucket.free_list, page as *mut DoublyListHead)
        };
        return;
    }

    toggle_bitmap(unsafe { &mut *bucket.bitmap }, bit_idx(page, order));

    if unsafe { &mut *bucket.bitmap }[bit_idx(page, order)] {
        // Buddy is not freed

        unsafe {
            DoublyListHead::insert_after(&raw mut bucket.free_list, page as *mut DoublyListHead)
        };
    } else {
        // Buddy is freed

        // Calculate buddy address
        let buddy = (page.addr() ^ (PAGE_SIZE << order)) as *mut u8;

        // Remove buddy from free list
        unsafe { DoublyListHead::delete(buddy as *mut DoublyListHead) };

        // Merge buddies
        let merged = if page.addr() < buddy.addr() {
            page
        } else {
            buddy
        };
        unsafe { free_pages_order(merged, order + 1) };
    }
}

#[inline]
pub unsafe fn alloc_pages(num_pages: usize) -> *mut u8 {
    unsafe { alloc_pages_order(log2_ceil(num_pages)) }
}

#[inline]
pub unsafe fn free_pages(ptr: *mut u8, num_pages: usize) {
    unsafe { free_pages_order(ptr, log2_ceil(num_pages)) }
}
