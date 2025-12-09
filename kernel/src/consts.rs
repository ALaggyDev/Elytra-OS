/// Base address where physical memory is mapped (direct mapping)
pub const PHYS_MEM_OFFSET: usize = 0xffff800000000000;

/// Upper limit of userspace address space.
pub const USERSPACE_LIMIT: usize = 0xffff800000000000;

/// Base address where the kernel is loaded (this is set mainly for better debugging in gdb)
pub const KERNEL_OFFSET: usize = 0xffffffff80000000;

pub const PAGE_SIZE: usize = 4096;
