use crate::{helper::add_within_bounds, user::elf_structure::*};

// TODO: What happens if the structs are not aligned?
// This code is bad, fix it later

pub struct ElfParser<'a> {
    buf: &'a [u8],
}

impl<'a> ElfParser<'a> {
    pub fn parse(buf: &'a [u8]) -> Result<Self, ()> {
        easy_assert(buf.len() >= size_of::<ElfHeader>())?;

        let parser = Self { buf };
        let header = parser.get_header();

        easy_assert(&header.e_ident[0..4] == b"\x7FELF")?; // Check ELF magic number
        easy_assert(header.e_ident[4] == 2)?; // Only support 64-bit ELF
        easy_assert(header.e_ident[5] == 1)?; // Only support little-endian
        easy_assert(header.e_ident[6] == 1)?; // Only support ELF version 1

        easy_assert(header.e_type == ElfType::Executable)?; // Only support executable files
        easy_assert(header.e_machine == ElfMachine::x86_64)?; // Only support x86_64

        Ok(parser)
    }

    pub fn get_buf(&self) -> &'a [u8] {
        self.buf
    }

    pub fn get_header(&self) -> &ElfHeader {
        unsafe { &*(self.buf.as_ptr() as *const ElfHeader) }
    }

    pub fn get_program_header(&self, index: usize) -> Result<&ElfProgramHeader, ()> {
        let header = self.get_header();
        easy_assert(index < header.e_phnum as usize)?;

        let ph_offset = header.e_phoff as usize + index * header.e_phentsize as usize;
        add_within_bounds(ph_offset, size_of::<ElfProgramHeader>(), self.buf.len()).ok_or(())?;

        let ph = unsafe { &*(self.buf.as_ptr().add(ph_offset) as *const ElfProgramHeader) };
        Ok(ph)
    }

    pub fn get_section_header(&self, index: usize) -> Result<&ElfSectionHeader, ()> {
        let header = self.get_header();
        easy_assert(index < header.e_shnum as usize)?;

        let sh_offset = header.e_shoff as usize + index * header.e_shentsize as usize;
        add_within_bounds(sh_offset, size_of::<ElfSectionHeader>(), self.buf.len()).ok_or(())?;

        let sh = unsafe { &*(self.buf.as_ptr().add(sh_offset) as *const ElfSectionHeader) };
        Ok(sh)
    }
}

fn easy_assert(cond: bool) -> Result<(), ()> {
    if cond { Ok(()) } else { Err(()) }
}
