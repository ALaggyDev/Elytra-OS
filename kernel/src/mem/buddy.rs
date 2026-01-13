use core::{mem::MaybeUninit, ptr, slice};

use bitvec::slice::BitSlice;
use spin::Mutex;

use crate::{
    consts::PAGE_SIZE,
    helper::{align_up, log2_ceil, log2_floor},
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

unsafe impl Send for BuddyAllocator {}
unsafe impl Sync for BuddyAllocator {}

// This is really annoying, because when creating DoublyListHead, the next and prev pointers point to itself (self-referential struct).
// Here is a hacky way to initialize it. Hopefully I can find a better way in the future.

pub static BUDDY_ALLOCATOR: Mutex<BuddyAllocator> =
    Mutex::new(unsafe { MaybeUninit::zeroed().assume_init() });

// We can't initialize the buddy allocator in Rust style, because of the self-referential issue :(
pub unsafe fn init(memory: *mut [u8]) {
    let mut allocator = BUDDY_ALLOCATOR.lock();

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

impl BuddyAllocator {
    pub unsafe fn alloc_pages_order(&mut self, order: usize) -> *mut u8 {
        assert!(order <= MAX_ORDER);

        // Search for a free block in the free list
        if unsafe { !DoublyListHead::is_empty(&raw mut self.buckets[order].free_list) } {
            // Found a free block
            let page = self.buckets[order].free_list.next;

            // Remove the block from the free list
            unsafe { DoublyListHead::delete(page) };

            // Toggle the bitmap
            if order != MAX_ORDER {
                self.toggle_bitmap(order, page as *mut u8);
            }

            page as *mut u8
        } else if order == MAX_ORDER {
            // Allocate a new block from the memory pool

            if self.used + SIZE_OF_MAX_ORDER > self.memory.len() {
                return ptr::null_mut();
            }

            let page = unsafe { (self.memory as *mut u8).add(self.used) };
            self.used += SIZE_OF_MAX_ORDER;

            page
        } else {
            // Otherwise, try to split a larger block
            let buddies = unsafe { self.alloc_pages_order(order + 1) };
            if buddies.is_null() {
                return ptr::null_mut();
            }

            // Calculate the address of the buddy block
            let buddy = unsafe { buddies.add(PAGE_SIZE << order) };

            // Insert the buddy into the free list
            unsafe {
                DoublyListHead::insert_after(
                    &raw mut self.buckets[order].free_list,
                    buddy as *mut DoublyListHead,
                )
            };

            // Toggle the bitmap
            self.toggle_bitmap(order, buddy);

            buddies
        }
    }

    pub unsafe fn free_pages_order(&mut self, page: *mut u8, order: usize) {
        assert!(order <= MAX_ORDER);

        if order == MAX_ORDER {
            // Just add it to the free list
            unsafe {
                DoublyListHead::insert_after(
                    &raw mut self.buckets[order].free_list,
                    page as *mut DoublyListHead,
                )
            };
            return;
        }

        self.toggle_bitmap(order, page);

        if self.get_bitmap(order, page) {
            // Buddy is not freed

            unsafe {
                DoublyListHead::insert_after(
                    &raw mut self.buckets[order].free_list,
                    page as *mut DoublyListHead,
                )
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
            unsafe { self.free_pages_order(merged, order + 1) };
        }
    }

    // Get bit index for bitmap.
    fn bit_idx(&self, page: *mut u8, order: usize) -> usize {
        let offset = page as usize - self.memory.addr();
        (offset / PAGE_SIZE) >> (order + 1)
    }

    // Get bitmap at index.
    fn get_bitmap(&self, order: usize, page: *mut u8) -> bool {
        let bitmap = unsafe { &*self.buckets[order].bitmap };
        bitmap[self.bit_idx(page, order)]
    }

    // Toggle bitmap at index.
    fn toggle_bitmap(&mut self, order: usize, page: *mut u8) {
        let bitmap = unsafe { &mut *self.buckets[order].bitmap };
        let idx = self.bit_idx(page, order);
        bitmap.set(idx, !bitmap[idx]);
    }
}

pub unsafe fn alloc_pages_order(order: usize) -> *mut u8 {
    let mut allocator = BUDDY_ALLOCATOR.lock();

    unsafe { allocator.alloc_pages_order(order) }
}

pub unsafe fn free_pages_order(page: *mut u8, order: usize) {
    let mut allocator = BUDDY_ALLOCATOR.lock();

    unsafe { allocator.free_pages_order(page, order) }
}

#[inline]
pub unsafe fn alloc_pages(num_pages: usize) -> *mut u8 {
    unsafe { alloc_pages_order(log2_ceil(num_pages)) }
}

#[inline]
pub unsafe fn free_pages(ptr: *mut u8, num_pages: usize) {
    unsafe { free_pages_order(ptr, log2_ceil(num_pages)) }
}

#[inline]
pub unsafe fn alloc_pages_order_panic(order: usize) -> *mut u8 {
    let ptr = unsafe { alloc_pages_order(order) };
    if ptr.is_null() {
        panic!("Buddy allocator: Out of memory");
    }
    ptr
}

#[inline]
pub unsafe fn alloc_pages_panic(num_pages: usize) -> *mut u8 {
    let ptr = unsafe { alloc_pages(num_pages) };
    if ptr.is_null() {
        panic!("Buddy allocator: Out of memory");
    }
    ptr
}

// Calculate the page order required for a given size.
// Equivalent to log2_ceil(size.div_ceil(PAGE_SIZE)), but more efficient.
#[inline]
pub fn calculate_order(size: usize) -> usize {
    // LLVM doesn't actually optimize the case when size == 1 well...
    size.strict_sub(1)
        .checked_ilog2()
        .map_or(0, |val| val as usize)
        .saturating_sub(log2_floor(PAGE_SIZE) - 1)
}
