// All user space memory accesses MUST be done via these functions.
//
// Note: We don't currently prevent memory exhaustion attacks. Fix this later.

use core::{mem::MaybeUninit, ptr};

use alloc::boxed::Box;

use crate::{consts::USERSPACE_LIMIT, helper::add_within_bounds};

/// Copy data from user space to kernel space.
///
/// # Safety
/// Caller must ensure that `dest` points to a valid kernel space memory region of at least `len` bytes.
///
/// The physical addresses of dest and src must not overlap.
pub unsafe fn copy_from_user(dest: *mut u8, src: *const u8, len: usize) -> Result<(), ()> {
    // Validate user space address range
    if add_within_bounds(src as usize, len, USERSPACE_LIMIT).is_none() {
        return Err(());
    }

    // TODO: This is deeply unsound. Fix it later.
    // 1. We did not handle page faults.
    // 2. We ignore page protections.
    unsafe { ptr::copy_nonoverlapping(src, dest, len) };

    Ok(())
}

/// Copy data from kernel space to user space.
///
/// # Safety
/// Caller must ensure that `src` points to a valid kernel space memory region of at least `len` bytes.
///
/// The physical addresses of dest and src must not overlap.
pub unsafe fn copy_to_user(dest: *mut u8, src: *const u8, len: usize) -> Result<(), ()> {
    // Validate user space address range
    if add_within_bounds(dest as usize, len, USERSPACE_LIMIT).is_none() {
        return Err(());
    }

    // TODO: This is deeply unsafe. Fix it later.
    // 1. We did not handle page faults.
    // 2. We ignore page protections.
    unsafe { ptr::copy_nonoverlapping(src, dest, len) };

    Ok(())
}

/// Copy data from user space to kernel space, returning the copied value.
/// This is suitable for normal sized types that fit on the stack.
pub unsafe fn copy_from_user_sized<T>(src: *const T) -> Result<T, ()> {
    unsafe {
        let mut val: MaybeUninit<T> = MaybeUninit::uninit();

        copy_from_user(
            val.as_mut_ptr() as *mut u8,
            src as *const u8,
            size_of::<T>(),
        )?;

        Ok(val.assume_init())
    }
}

/// Copy data from user space to kernel space, allocating a `Box<[T]>` for the destination.
/// This is suitable for slices like `[u8]`.
pub unsafe fn copy_from_user_slice<T>(src: *const [T]) -> Result<Box<[T]>, ()> {
    unsafe {
        let mut boxed: Box<[MaybeUninit<T>]> = Box::new_uninit_slice(src.len());

        copy_from_user(
            boxed.as_mut_ptr() as *mut u8,
            src as *const u8,
            src.len() * size_of::<T>(),
        )?;

        Ok(boxed.assume_init())
    }
}
