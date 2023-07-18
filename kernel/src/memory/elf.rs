use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::fmt::{Debug, Formatter};

use page_table::table::Sv39PageTable;
use xmas_elf::ElfFile;
use xmas_elf::sections::SectionData;
use xmas_elf::symbol_table::Entry;

use crate::memory::PageAllocator;

#[derive(Debug)]
pub enum ELFError {
    NotELF,
    FileBreak,
    NotSupported,
    NoLoadableSegment,
    NoStackSegment,
    NoEntrySegment,
    RelocationError,
    DynsymNotFind,
}


impl Debug for ELFInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!(
            "ELFInfo {{ address_space: {:#x?}, entry: {:#x}, stack_top: {:#x} }}",
            self.address_space.root_paddr().as_usize() >> 12,
            self.entry,
            self.stack_top
        ))
    }
}

pub struct ELFInfo {
    pub address_space: Sv39PageTable<PageAllocator>,
    pub entry: usize,
    pub stack_top: usize,
    pub heap_bottom: usize,
    pub ph_num: usize,
    pub ph_entry_size: usize,
    pub ph_drift: usize,
    pub tls: usize,
    pub bias: usize,
    pub name: String,
}

pub trait ELFReader {
    fn build_elf(&mut self) -> Result<ELFInfo, ELFError>;
    fn relocate(&self, bias: usize) -> Result<Vec<(usize, usize)>, ELFError>;
}

impl ELFReader for ElfFile<'_> {
    fn build_elf(&mut self) -> Result<ELFInfo, ELFError> {
        Err(ELFError::NotSupported)
    }
    fn relocate(&self, bias: usize) -> Result<Vec<(usize, usize)>, ELFError> {
        let mut res = vec![];
        let data = self
            .find_section_by_name(".rela.dyn")
            .ok_or(ELFError::RelocationError)?
            .get_data(self)
            .map_err(|_| ELFError::RelocationError)?;
        let entries = match data {
            SectionData::Rela64(entries) => entries,
            _ => return Err(ELFError::RelocationError),
        };
        let dynsym = match self
            .find_section_by_name(".dynsym")
            .ok_or(ELFError::DynsymNotFind)?
            .get_data(self)
            .map_err(|_| ELFError::DynsymNotFind)?
        {
            SectionData::DynSymbolTable64(dsym) => Ok(dsym),
            _ => Err(ELFError::DynsymNotFind),
        }?;
        for entry in entries.iter() {
            const REL_GOT: u32 = 6;
            const REL_PLT: u32 = 7;
            const REL_RELATIVE: u32 = 8;
            const R_RISCV_64: u32 = 2;
            const R_RISCV_RELATIVE: u32 = 3;
            match entry.get_type() {
                REL_GOT | REL_PLT | R_RISCV_64 => {
                    let dynsym = &dynsym[entry.get_symbol_table_index() as usize];
                    let symval = if dynsym.shndx() == 0 {
                        let name = dynsym.get_name(self).map_err(|_| ELFError::DynsymNotFind)?;
                        panic!("need to find symbol: {:?}", name);
                    } else {
                        bias + dynsym.value() as usize
                    };
                    let value = symval + entry.get_addend() as usize;
                    let addr = bias + entry.get_offset() as usize;
                    res.push((addr, value))
                }
                REL_RELATIVE | R_RISCV_RELATIVE => {
                    let value = bias + entry.get_addend() as usize;
                    let addr = bias + entry.get_offset() as usize;
                    res.push((addr, value))
                }
                t => unimplemented!("unknown type: {}", t),
            }
        }

        //
        let data = self
            .find_section_by_name(".rela.plt")
            .ok_or(ELFError::RelocationError)?
            .get_data(self)
            .map_err(|_| ELFError::RelocationError)?;
        let entries = match data {
            SectionData::Rela64(entries) => entries,
            _ => return Err(ELFError::RelocationError),
        };
        for entry in entries.iter() {
            match entry.get_type() {
                5 => {
                    let dynsym = &dynsym[entry.get_symbol_table_index() as usize];
                    let symval = if dynsym.shndx() == 0 {
                        let name = dynsym.get_name(self)
                            .map_err(|_| ELFError::DynsymNotFind)?;
                        panic!("symbol not found: {:?}", name);
                    } else {
                        dynsym.value() as usize
                    };
                    let value = bias + symval;
                    let addr = bias + entry.get_offset() as usize;
                    res.push((addr, value))
                }
                t => panic!("[kernel] unknown entry, type = {}", t),
            }
        }
        Ok(res)
    }
}