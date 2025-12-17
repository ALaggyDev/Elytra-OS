use core::ptr::{copy_nonoverlapping, null_mut};

use alloc::vec::Vec;
use arbitrary_int::traits::Integer;

use crate::{
    consts::{PAGE_SIZE, USERSPACE_LIMIT},
    helper::{add_within_bounds, align_down, align_up, log2_ceil, p2v, v2p},
    mem::{
        buddy::{alloc_pages_order_panic, alloc_pages_panic, free_pages, free_pages_order},
        page_table::{
            PageDirectory, PageDirectoryEntry, VirtAddr, resolve_virt_addr,
            set_active_page_directory,
        },
    },
    user::{elf_parser::ElfParser, elf_structure::ElfProgramHeaderType},
};

pub static mut KERNEL_P4_TABLE: *mut PageDirectory = null_mut();

#[derive(Debug)]
pub struct VirtRegion {
    pub start: usize,
    pub len: usize,
    pub writable: bool,
    pub executable: bool,

    // From buddy allocator
    backing_pages: *mut u8,
    backing_order: usize,
}

// A userspace address space.
#[derive(Debug)]
pub struct AddressSpace {
    p4_table: *mut PageDirectory,
    virt_regions: Vec<VirtRegion>,
    allocated_tables: Vec<*mut u8>,
}

impl AddressSpace {
    /// Create a new AddressSpace with a new P4 page table.
    pub fn new() -> Self {
        unsafe {
            let p4_table = alloc_pages_panic(1) as *mut PageDirectory;
            p4_table.write_bytes(0, 1);

            Self {
                p4_table,
                virt_regions: vec![],
                allocated_tables: vec![p4_table as *mut u8],
            }
        }
    }

    /// Map all kernel space pages.
    pub fn map_kernel_pages(&mut self) {
        unsafe {
            assert!(!KERNEL_P4_TABLE.is_null());

            for i in 256..512 {
                (*self.p4_table).0[i] = (*KERNEL_P4_TABLE).0[i];
            }
        }
    }

    /// Get the P4 page table pointer.
    pub fn p4_table(&self) -> *mut PageDirectory {
        self.p4_table
    }

    /// Resolve a virtual address to a physical address.
    /// Page entry permissions are ignored.
    pub fn resolve_virt_addr(&self, virt_addr: usize) -> Option<usize> {
        unsafe { resolve_virt_addr(self.p4_table, virt_addr) }
    }

    /// Test if a region does not overlap with existing regions and is within userspace bounds.
    pub fn check_region_no_overlap(&self, start: usize, len: usize) -> bool {
        // Forbid addresses not within USERSPACE_LIMIT. We block the first and last page in the userspace too.
        if start == 0 {
            return false;
        }
        let Some(end) = add_within_bounds(start, len, USERSPACE_LIMIT - PAGE_SIZE) else {
            return false;
        };

        for region in &self.virt_regions {
            let region_end = region.start + region.len;
            if !(end <= region.start || start >= region_end) {
                return false;
            }
        }
        true
    }

    /// Add a virtual region. Returns the page pointer if successful. The pages will be zeroed.
    pub fn add_virt_region(
        &mut self,
        start: usize,
        len: usize,
        writable: bool,
        executable: bool,
    ) -> Result<*mut u8, ()> {
        let start = align_down(start, PAGE_SIZE);
        let len = align_up(len, PAGE_SIZE);

        if !self.check_region_no_overlap(start, len) {
            return Err(());
        }

        // Allocate some pages.
        let num_order = log2_ceil(len / PAGE_SIZE);
        let pages = unsafe { alloc_pages_order_panic(num_order) };
        unsafe { pages.write_bytes(0, len) };

        // Map pages.
        for offset in (0..len).step_by(PAGE_SIZE) {
            self.map_virt_addr(
                start + offset,
                v2p(unsafe { pages.add(offset) } as usize),
                writable,
                executable,
            );
        }

        // Record region.
        self.virt_regions.push(VirtRegion {
            start,
            len,
            writable,
            executable,

            backing_pages: pages,
            backing_order: num_order,
        });

        Ok(pages)
    }

    unsafe fn get_or_create_page_table(
        &mut self,
        page_table: *mut PageDirectory,
        index: usize,
    ) -> *mut PageDirectory {
        unsafe {
            let entry = (*page_table).0[index];
            if entry.present() {
                p2v(entry.addr() as usize) as *mut PageDirectory
            } else {
                let new_table = alloc_pages_panic(1) as *mut PageDirectory;
                self.allocated_tables.push(new_table as *mut u8);
                new_table.write_bytes(0, 1);

                let new_entry = PageDirectoryEntry::ZERO
                    .with_present(true)
                    .with_writable(true)
                    .with_user_accessible(true)
                    .with_addr(v2p(new_table as usize) as u64);
                (*page_table).0[index] = new_entry;

                new_table
            }
        }
    }

    // Map a virtual address (aligned to PAGE_SIZE) to a physical address.
    fn map_virt_addr(
        &mut self,
        virt_addr: usize,
        phys_addr: usize,
        writable: bool,
        executable: bool,
    ) {
        let virt_addr = VirtAddr::new_with_raw_value(virt_addr as u64);

        unsafe {
            let p3_table =
                self.get_or_create_page_table(self.p4_table, virt_addr.p4_index().as_usize());
            let p2_table = self.get_or_create_page_table(p3_table, virt_addr.p3_index().as_usize());
            let p1_table = self.get_or_create_page_table(p2_table, virt_addr.p2_index().as_usize());

            let p1_entry = PageDirectoryEntry::ZERO
                .with_present(true)
                .with_writable(writable)
                .with_user_accessible(true)
                .with_execute_disable(!executable)
                .with_addr(phys_addr as u64);
            (*p1_table).0[virt_addr.p1_index().as_usize()] = p1_entry;
        }
    }

    /// Switch to this address space.
    pub unsafe fn switch_to_this(&self) {
        unsafe { set_active_page_directory(self.p4_table) };
    }

    /// Map ELF segments into the address space.
    pub fn map_elf_segments(&mut self, parser: &ElfParser) -> Result<(), ()> {
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
            let region = self.add_virt_region(vaddr, mem_size, writable, executable)?;

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

        Ok(())
    }
}

impl Drop for AddressSpace {
    fn drop(&mut self) {
        // Deallocate page tables.
        for &table in &self.allocated_tables {
            unsafe { free_pages(table, 1) };
        }

        // Deallocate backing pages.
        for region in &self.virt_regions {
            unsafe { free_pages_order(region.backing_pages, region.backing_order) };
        }
    }
}
