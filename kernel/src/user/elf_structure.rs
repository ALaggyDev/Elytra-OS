// TODO: What happens if the structs are not aligned?

#[derive(Debug)]
#[repr(C)]
pub struct ElfHeader {
    pub e_ident: [u8; 16],     // Magic number and other info
    pub e_type: ElfType,       // Object file type
    pub e_machine: ElfMachine, // Architecture
    pub e_version: u32,        // Object file version
    pub e_entry: u64,          // Entry point virtual address
    pub e_phoff: u64,          // Program header table file offset
    pub e_shoff: u64,          // Section header table file offset
    pub e_flags: u32,          // Processor-specific flags
    pub e_ehsize: u16,         // ELF header size in bytes
    pub e_phentsize: u16,      // Program header table entry size
    pub e_phnum: u16,          // Program header table entry count
    pub e_shentsize: u16,      // Section header table entry size
    pub e_shnum: u16,          // Section header table entry count
    pub e_shstrndx: u16,       // Section header string table index
}

#[derive(Debug)]
#[repr(C)]
pub struct ElfProgramHeader {
    pub p_type: ElfProgramHeaderType, // Segment type
    pub p_flags: u32,                 // Segment flags
    pub p_offset: u64,                // Segment file offset
    pub p_vaddr: u64,                 // Segment virtual address
    pub p_paddr: u64,                 // Segment physical address
    pub p_filesz: u64,                // Segment size in file
    pub p_memsz: u64,                 // Segment size in memory
    pub p_align: u64,                 // Segment alignment
}

#[derive(Debug)]
#[repr(C)]
pub struct ElfSectionHeader {
    pub sh_name: u32,                  // Section name (string table index)
    pub sh_type: ElfSectionHeaderType, // Section type
    pub sh_flags: u64,                 // Section flags
    pub sh_addr: u64,                  // Section virtual address at execution
    pub sh_offset: u64,                // Section file offset
    pub sh_size: u64,                  // Section size in bytes
    pub sh_link: u32,                  // Link to another section
    pub sh_info: u32,                  // Additional section information
    pub sh_addralign: u64,             // Section alignment
    pub sh_entsize: u64,               // Entry size if section holds a table
}

// Create a "enum" where only some variants have names, but all bit patterns are still valid
macro_rules! open_enum {
    ($(#[$attr:meta])* $vis:vis struct $name:ident($type:ty) {
        $($var_name:ident = $var_value:expr),* $(,)?
    }) => {
        $(#[$attr])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        #[repr(transparent)]
        $vis struct $name(pub $type);

        impl $name {
            $(
                #[allow(non_upper_case_globals)]
                pub const $var_name: Self = Self($var_value);
            )*
        }
    };
}

open_enum! {
    pub struct ElfType(u16) {
        None = 0,
        Relocatable = 1,
        Executable = 2,
        SharedObject = 3,
        Core = 4,
    }
}

open_enum! {
    pub struct ElfMachine(u16) {
        None = 0,
        Sparc = 0x2,
        x86 = 0x3,
        MIPS = 0x8,
        PowerPC = 0x14,
        ARM = 0x28,
        Sparc64 = 0x2b,
        IA64 = 0x32,
        x86_64 = 0x3e,
        AArch64 = 0xb7,
        RiscV = 0xf3,
    }
}

open_enum! {
    pub struct ElfProgramHeaderType(u32) {
        Null = 0,
        Load = 1,
        Dynamic = 2,
        Interp = 3,
        Note = 4,
        Shlib = 5,
        Phdr = 6,
        Tls = 7,
    }
}

open_enum! {
    pub struct ElfSectionHeaderType(u32) {
        Null = 0,
        Progbits = 1,
        Symtab = 2,
        Strtab = 3,
        Rela = 4,
        Hash = 5,
        Dynamic = 6,
        Note = 7,
        Nobits = 8,
        Rel = 9,
        Shlib = 10,
        Dynsym = 11,
        InitArray = 14,
        FiniArray = 15,
        PreinitArray = 16,
        Group = 17,
        SymtabShndx = 18,
    }
}
