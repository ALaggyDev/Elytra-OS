//! A basic slab allocator.
//!
//! This allocator is so basic that perhaps it shouldn't even be called a slab allocator haha.
//! For each cache size, it simply maintains a huge freelist of freed objects among all slabs, and allocates from there.
//! There is no per slab book-keeping. So unused slabs cannot be freed back to the buddy allocator.

use core::{
    alloc::{GlobalAlloc, Layout},
    cell::UnsafeCell,
    cmp::{max, min},
    ptr::{self, null_mut},
};

use crate::{
    consts::PAGE_SIZE,
    helper::log2_floor,
    mem::buddy::{alloc_pages_order, free_pages_order},
    primitives::SinglyListHead,
};

#[derive(Debug)]
struct Cache {
    obj_size: usize,   // Size of each object
    slab_order: usize, // Order of pages per slab

    freelist: SinglyListHead, // Freelist of freed objects
}

impl Cache {
    const fn new(obj_size: usize, num_pages: usize) -> Self {
        Cache {
            obj_size,
            slab_order: log2_floor(num_pages),
            freelist: SinglyListHead::new(),
        }
    }
}

#[derive(Debug)]
struct SlabAllocator {
    // Caches MUST be sorted by obj_size in ascending order
    caches: [Cache; 8],
}

impl SlabAllocator {
    const fn new() -> Self {
        SlabAllocator {
            caches: [
                Cache::new(16, 1),   // 16 bytes, 1 page per slab
                Cache::new(32, 1),   // 32 bytes, 1 page per slab
                Cache::new(64, 1),   // 64 bytes, 1 page per slab
                Cache::new(128, 1),  // 128 bytes, 1 page per slab
                Cache::new(256, 2),  // 256 bytes, 2 page per slab
                Cache::new(512, 2),  // 512 bytes, 2 page per slab
                Cache::new(1024, 2), // 1024 bytes, 2 page per slab
                Cache::new(2048, 2), // 2048 bytes, 2 page per slab
            ],
        }
    }

    // Calculate the page order required for a given size.
    // Equivalent to log2_ceil(size.div_ceil(PAGE_SIZE)), but more efficient.
    fn calculate_order(size: usize) -> usize {
        // LLVM doesn't actually optimize the case when size == 1 well...
        size.strict_sub(1)
            .checked_ilog2()
            .map_or(0, |val| val as usize)
            .saturating_sub(log2_floor(PAGE_SIZE) - 1)
    }

    fn find_cache(&self, size: usize) -> Option<&Cache> {
        self.caches.iter().find(|cache| size <= cache.obj_size)
    }

    fn find_cache_mut(&mut self, size: usize) -> Option<&mut Cache> {
        self.caches.iter_mut().find(|cache| size <= cache.obj_size)
    }

    fn check_same_basket(&self, old_size: usize, new_size: usize) -> bool {
        match (self.find_cache(old_size), self.find_cache(new_size)) {
            (Some(old_cache), Some(new_cache)) => old_cache.obj_size == new_cache.obj_size,
            (None, None) => Self::calculate_order(old_size) == Self::calculate_order(new_size),
            _ => false,
        }
    }

    // Allocate a buffer of at least `size` bytes.
    pub unsafe fn alloc(&mut self, size: usize) -> *mut u8 {
        let cache = self.find_cache_mut(size);

        if let Some(cache) = cache {
            // Allocate from the slab allocator.

            let obj = unsafe { cache.freelist.pop() };
            if !obj.is_null() {
                // Found a free object. Return it directly.
                obj as *mut u8
            } else {
                // No free object, allocate a new slab.
                let slab_ptr = unsafe { alloc_pages_order(cache.slab_order) };
                if slab_ptr.is_null() {
                    return null_mut();
                }

                // Split the slab into objects and push them to the freelist.
                let slab_size = PAGE_SIZE << cache.slab_order;
                let num_objs = slab_size / cache.obj_size;
                let mut prev_obj_ptr = null_mut();
                for i in 0..num_objs {
                    unsafe {
                        let obj_ptr = slab_ptr.add(i * cache.obj_size) as *mut SinglyListHead;
                        *obj_ptr = SinglyListHead { next: prev_obj_ptr };
                        prev_obj_ptr = obj_ptr;
                    }
                }

                cache.freelist = SinglyListHead { next: prev_obj_ptr };

                // Pop one object to return.
                let obj = unsafe { cache.freelist.pop() };
                obj as *mut u8
            }
        } else {
            // Allocate from the buddy allocator.

            unsafe { alloc_pages_order(Self::calculate_order(size)) }
        }
    }

    // Free an allocated buffer of given size.
    pub unsafe fn dealloc(&mut self, ptr: *mut u8, size: usize) {
        let cache = self.find_cache_mut(size);

        if let Some(cache) = cache {
            // Free to the slab allocator.

            unsafe { cache.freelist.insert_after(ptr as *mut _) }
        } else {
            // Free to the buddy allocator.

            unsafe { free_pages_order(ptr, Self::calculate_order(size)) }
        }
    }

    // Reallocate a buffer to a new size.
    pub unsafe fn realloc(&mut self, ptr: *mut u8, old_size: usize, new_size: usize) -> *mut u8 {
        if self.check_same_basket(old_size, new_size) {
            // Same basket, no need to reallocate.
            return ptr;
        }

        // Different basket, allocate new buffer and copy data.
        let new_ptr = unsafe { self.alloc(new_size) };
        if new_ptr.is_null() {
            return null_mut();
        }

        // Copy the data from old buffer to new buffer and free old buffer.
        unsafe {
            ptr::copy_nonoverlapping(ptr, new_ptr, min(old_size, new_size));
            self.dealloc(ptr, old_size);
        }

        new_ptr
    }
}

#[derive(Debug)]
pub struct SlabAllocatorWrapper(UnsafeCell<SlabAllocator>);

unsafe impl Sync for SlabAllocatorWrapper {}

unsafe impl GlobalAlloc for SlabAllocatorWrapper {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = max(layout.size(), layout.align());

        let allocator = unsafe { &mut *self.0.get() };
        unsafe { allocator.alloc(size) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let size = max(layout.size(), layout.align());

        let allocator = unsafe { &mut *self.0.get() };
        unsafe { allocator.dealloc(ptr, size) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let old_size = max(layout.size(), layout.align());
        let new_size = max(new_size, layout.align());

        let allocator = unsafe { &mut *self.0.get() };
        unsafe { allocator.realloc(ptr, old_size, new_size) }
    }
}

#[global_allocator]
pub static SLAB_ALLOCATOR: SlabAllocatorWrapper =
    SlabAllocatorWrapper(UnsafeCell::new(SlabAllocator::new()));
