use alloc::vec;
use alloc::vec::Vec;
use config::FRAME_SIZE;
use constants::{AlienError, AlienResult};
use log::{info, trace};
use mem::{alloc_free_region, map_region_to_kernel, query_kernel_space, MappingFlags, VirtAddr};
use xmas_elf::program::Type;
use xmas_elf::sections::{Rela, SectionData};
use xmas_elf::symbol_table::{DynEntry64, Entry};
use xmas_elf::{ElfFile, P64};

pub struct DomainLoader {
    entry: usize,
}

impl DomainLoader {
    pub fn new() -> Self {
        Self { entry: 0 }
    }
    pub fn entry(&self) -> usize {
        self.entry
    }
    pub fn load(&mut self, elf_binary: &[u8]) -> AlienResult<()> {
        const ELF_MAGIC: [u8; 4] = [0x7f, b'E', b'L', b'F'];
        if elf_binary[0..4] != ELF_MAGIC {
            return Err(AlienError::EINVAL);
        }
        info!("Domain address:{:p}", elf_binary.as_ptr());
        let elf = ElfFile::new(elf_binary).unwrap();
        info!("Domain type:{:?}", elf.header.pt2.type_().as_type());
        let end_paddr = elf
            .program_iter()
            .filter(|ph| ph.get_type() == Ok(Type::Load))
            .last()
            .map(|x| x.virtual_addr() as usize + x.mem_size() as usize)
            .unwrap();
        let end_paddr = VirtAddr::from(end_paddr).align_up(FRAME_SIZE);
        // alloc free page to map elf
        let region_start = alloc_free_region(end_paddr.as_usize()).unwrap();
        info!(
            "region range:{:#x}-{:#x}",
            region_start,
            region_start + end_paddr.as_usize()
        );
        elf.program_iter()
            .filter(|ph| ph.get_type() == Ok(Type::Load))
            .for_each(|ph| {
                let start_vaddr = ph.virtual_addr() as usize + region_start;
                let end_vaddr = start_vaddr + ph.mem_size() as usize;
                let mut permission: MappingFlags = "VAD".into();
                let ph_flags = ph.flags();
                if ph_flags.is_read() {
                    permission |= MappingFlags::R;
                }
                if ph_flags.is_write() {
                    permission |= MappingFlags::W;
                }
                if ph_flags.is_execute() {
                    permission |= MappingFlags::X;
                }
                let mut vaddr = VirtAddr::from(start_vaddr).align_down_4k().as_usize();
                let end_vaddr = VirtAddr::from(end_vaddr).align_up_4k().as_usize();
                let len = end_vaddr - vaddr;
                info!(
                    "map range: [{:#x}-{:#x}], memsize:{}, perm:{:?}",
                    vaddr,
                    vaddr + len,
                    ph.mem_size(),
                    permission
                );
                map_region_to_kernel(vaddr, len, permission).unwrap();
                let data =
                    &elf.input[ph.offset() as usize..(ph.offset() + ph.file_size()) as usize];
                let mut page_offset = start_vaddr & (FRAME_SIZE - 1);
                let mut count = 0;
                while count < data.len() {
                    let paddr = query_kernel_space(vaddr).unwrap();
                    let len = core::cmp::min(FRAME_SIZE - page_offset, data.len() - count);
                    let dst_buf =
                        unsafe { core::slice::from_raw_parts_mut(paddr as *mut u8, FRAME_SIZE) };
                    dst_buf[page_offset..page_offset + len]
                        .copy_from_slice(&data[count..count + len]);
                    trace!("copy data to {:#x}-{:#x}", paddr, paddr + len);
                    vaddr += len;
                    page_offset = 0;
                    count += len;
                }
                assert_eq!(count, data.len());
            });

        if let Ok(res) = relocate_dyn(&elf, region_start) {
            info!("Relocate_dyn {} entries", res.len());
            res.into_iter().for_each(|kv| {
                trace!("relocate: {:#x} -> {:#x}", kv.0, kv.1);
                let addr = query_kernel_space(kv.0).unwrap();
                unsafe { (addr as *mut usize).write(kv.1) }
            });
            info!("Relocate_dyn done");
        }
        if let Ok(res) = relocate_plt(&elf, region_start) {
            info!("Relocate_plt");
            res.into_iter().for_each(|kv| {
                trace!("relocate: {:#x} -> {:#x}", kv.0, kv.1);
                let addr = query_kernel_space(kv.0).unwrap();
                unsafe { (addr as *mut usize).write(kv.1) }
            });
            info!("Relocate_plt done");
        }
        let entry = elf.header.pt2.entry_point() as usize + region_start;
        info!("entry: {:#x}", entry);
        self.entry = entry;
        Ok(())
    }
}

fn relocate(
    region_start: usize,
    elf: &ElfFile,
    dynsym: &[DynEntry64],
    entry: &[Rela<P64>],
) -> Result<Vec<(usize, usize)>, &'static str> {
    let mut res = vec![];
    for entry in entry.iter() {
        match entry.get_type() {
            R_RISCV_64 => {
                trace!("dynsym: {:?}", dynsym);
                let dynsym = &dynsym[entry.get_symbol_table_index() as usize];
                let symval = if dynsym.shndx() == 0 {
                    let name = dynsym.get_name(&elf)?;
                    panic!("symbol not found: {:?}", name);
                } else {
                    dynsym.value() as usize
                };
                let value = symval + entry.get_addend() as usize;
                let addr = region_start + entry.get_offset() as usize;
                res.push((addr, value))
            }
            R_RISCV_RELATIVE => {
                let value = region_start + entry.get_addend() as usize;
                let addr = region_start + entry.get_offset() as usize;
                res.push((addr, value))
            }
            t => unimplemented!("unknown type: {}", t),
        }
    }
    Ok(res)
}

fn relocate_dyn(elf: &ElfFile, region_start: usize) -> Result<Vec<(usize, usize)>, &'static str> {
    let data = elf
        .find_section_by_name(".rela.dyn")
        .map(|h| h.get_data(&elf).unwrap())
        .ok_or("corrupted .rela.dyn")?;
    let entries = match data {
        SectionData::Rela64(entries) => entries,
        _ => return Err("bad .rela.dyn"),
    };
    let dynsym = match elf
        .find_section_by_name(".dynsym")
        .ok_or(".dynsym not found")?
        .get_data(&elf)
        .map_err(|_| "corrupted .dynsym")?
    {
        SectionData::DynSymbolTable64(dsym) => Ok(dsym),
        _ => Err("bad .dynsym"),
    }?;
    relocate(region_start, elf, dynsym, entries)
}

fn relocate_plt(elf: &ElfFile, region_start: usize) -> Result<Vec<(usize, usize)>, &'static str> {
    let mut res = vec![];
    let data = elf
        .find_section_by_name(".rela.plt")
        .ok_or(".rela.plt not found")?
        .get_data(&elf)
        .map_err(|_| "corrupted .rela.plt")?;
    let entries = match data {
        SectionData::Rela64(entries) => entries,
        _ => return Err("bad .rela.plt"),
    };
    let dynsym = match elf
        .find_section_by_name(".dynsym")
        .ok_or(".dynsym not found")?
        .get_data(&elf)
        .map_err(|_| "corrupted .dynsym")?
    {
        SectionData::DynSymbolTable64(dsym) => Ok(dsym),
        _ => Err("bad .dynsym"),
    }?;
    for entry in entries.iter() {
        match entry.get_type() {
            R_RISCV_JUMP_SLOT => {
                let dynsym = &dynsym[entry.get_symbol_table_index() as usize];
                // let symval = if dynsym.shndx() == 0 {
                //     let name = dynsym.get_name(&elf)?;
                //     panic!("symbol not found: {:?}", name);
                // } else {
                //     dynsym.value() as usize
                // };
                let symval = dynsym.value() as usize;
                trace!("dynsym: {:?}", dynsym);
                let value = region_start + symval + entry.get_addend() as usize;
                let addr = region_start + entry.get_offset() as usize;
                res.push((addr, value))
            }
            t => panic!("[kernel] unknown entry, type = {}", t),
        }
    }
    Ok(res)
}

const R_RISCV_64: u32 = 2;
const R_RISCV_RELATIVE: u32 = 3;
const R_RISCV_JUMP_SLOT: u32 = 5;

// /* RISC-V relocations.  */
// #define R_RISCV_NONE             0
// #define R_RISCV_32               1
// #define R_RISCV_64               2
// #define R_RISCV_RELATIVE         3
// #define R_RISCV_COPY             4
// #define R_RISCV_JUMP_SLOT        5
// #define R_RISCV_TLS_DTPMOD32     6
// #define R_RISCV_TLS_DTPMOD64     7
// #define R_RISCV_TLS_DTPREL32     8
// #define R_RISCV_TLS_DTPREL64     9
// #define R_RISCV_TLS_TPREL32     10
// #define R_RISCV_TLS_TPREL64     11
// #define R_RISCV_BRANCH          16
// #define R_RISCV_JAL             17
