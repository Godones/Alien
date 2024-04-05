use alloc::{boxed::Box, vec, vec::Vec};
use core::{
    fmt::{Debug, Formatter},
    ops::Range,
};

use config::FRAME_SIZE;
use constants::{AlienError, AlienResult};
use domain_helper::{DomainSyscall, SharedHeapAllocator};
use log::{debug, info, trace};
use mem::{alloc_free_region, MappingFlags, VirtAddr};
use xmas_elf::{
    program::Type,
    sections::{Rela, SectionData},
    symbol_table::{DynEntry64, Entry},
    ElfFile, P64,
};

pub struct DomainLoader {
    entry: usize,
    data: &'static [u8],
    phy_start: usize,
    regions: Vec<RegionMeta>,
}

impl Debug for DomainLoader {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("DomainLoader")
            .field("entry", &self.entry)
            .field("phy_start", &self.phy_start)
            .field("regions", &self.regions)
            .finish()
    }
}

#[derive(Debug)]
struct VmInfo {
    range: Range<usize>,
    permission: MappingFlags,
}

impl VmInfo {
    pub fn new(range: Range<usize>, permission: MappingFlags) -> Self {
        Self { range, permission }
    }
    pub fn range(&self) -> &Range<usize> {
        &self.range
    }
    pub fn permission(&self) -> MappingFlags {
        self.permission
    }
}

#[derive(Debug)]
struct RegionMeta {
    vm: VmInfo,
    vm_start: usize,
    data_offset: usize,
    data_size: usize,
}

impl DomainLoader {
    pub fn new(data: &'static [u8]) -> Self {
        Self {
            entry: 0,
            data,
            phy_start: 0,
            regions: vec![],
        }
    }
    pub fn data(&self) -> &'static [u8] {
        self.data
    }
    pub fn entry(&self) -> usize {
        self.entry
    }

    pub fn call<T: ?Sized>(&self, id: u64) -> Box<T> {
        type F<T> =
            fn(Box<dyn corelib::CoreFunction>, u64, Box<dyn rref::SharedHeapAlloc>) -> Box<T>;
        let main = unsafe { core::mem::transmute::<*const (), F<T>>(self.entry() as *const ()) };
        let syscall = Box::new(DomainSyscall);
        let heap = Box::new(SharedHeapAllocator);

        let syscall_ptr = Box::into_raw(syscall);
        let heap_ptr = Box::into_raw(heap);

        domain_helper::register_domain_syscall_resource(id, syscall_ptr as usize);
        domain_helper::register_domain_heap_resource(id, heap_ptr as usize);
        unsafe { main(Box::from_raw(syscall_ptr), id, Box::from_raw(heap_ptr)) }
    }

    fn load_program(&mut self, elf: &ElfFile) -> AlienResult<()> {
        elf.program_iter()
            .filter(|ph| ph.get_type() == Ok(Type::Load))
            .for_each(|ph| {
                let start_vaddr = ph.virtual_addr() as usize + self.phy_start;
                let end_vaddr = start_vaddr + ph.mem_size() as usize;
                let mut permission = MappingFlags::empty();
                let ph_flags = ph.flags();
                if ph_flags.is_read() {
                    permission |= MappingFlags::READ;
                }
                if ph_flags.is_write() {
                    permission |= MappingFlags::WRITE;
                }
                if ph_flags.is_execute() {
                    permission |= MappingFlags::EXECUTE;
                }
                let mut vaddr = VirtAddr::from(start_vaddr).align_down_4k().as_usize();
                let end_vaddr = VirtAddr::from(end_vaddr).align_up_4k().as_usize();
                let len = end_vaddr - vaddr;
                trace!(
                    "map range: [{:#x}-{:#x}], memsize:{}, perm:{:?}",
                    vaddr,
                    end_vaddr,
                    ph.mem_size(),
                    permission
                );
                mem::map_region_to_kernel(vaddr, len, permission).unwrap();
                // save region
                let meta = RegionMeta {
                    vm: VmInfo::new(vaddr..end_vaddr, permission),
                    data_offset: ph.offset() as usize,
                    data_size: ph.file_size() as usize,
                    vm_start: start_vaddr,
                };
                self.regions.push(meta);

                let data =
                    &elf.input[ph.offset() as usize..(ph.offset() + ph.file_size()) as usize];
                let mut page_offset = start_vaddr & (FRAME_SIZE - 1);
                let mut count = 0;
                while count < data.len() {
                    let paddr = mem::query_kernel_space(vaddr).unwrap();
                    // let paddr = vaddr;
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
        Ok(())
    }

    fn reload_program(&self, elf: &ElfFile) -> AlienResult<()> {
        for meta in &self.regions {
            // reload region which has write permission
            if meta.vm.permission().contains(MappingFlags::WRITE) {
                trace!(
                    "reload region: {:#x}-{:#x}, perm:{:?}",
                    meta.vm.range().start,
                    meta.vm.range().end,
                    meta.vm.permission()
                );
                let data = &elf.input[meta.data_offset..(meta.data_offset + meta.data_size)];
                let mut page_offset = meta.vm_start & (FRAME_SIZE - 1);
                let mut count = 0;
                let mut vaddr = meta.vm.range().start;
                while count < data.len() {
                    // let paddr = mem::query_kernel_space(vaddr).unwrap();
                    let paddr = vaddr;
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
            }
        }
        Ok(())
    }

    fn relocate_dyn(&self, elf: &ElfFile) -> AlienResult<()> {
        if let Ok(res) = relocate_dyn(&elf, self.phy_start) {
            trace!("Relocate_dyn {} entries", res.len());
            res.into_iter().for_each(|kv| {
                debug!("relocate: {:#x} -> {:#x}", kv.0, kv.1);
                let addr = mem::query_kernel_space(kv.0).unwrap();
                unsafe { (addr as *mut usize).write(kv.1) }
            });
            trace!("Relocate_dyn done");
        }
        if let Ok(res) = relocate_plt(&elf, self.phy_start) {
            trace!("Relocate_plt");
            res.into_iter().for_each(|kv| {
                trace!("relocate: {:#x} -> {:#x}", kv.0, kv.1);
                let addr = mem::query_kernel_space(kv.0).unwrap();
                unsafe { (addr as *mut usize).write(kv.1) }
            });
            trace!("Relocate_plt done");
        }
        Ok(())
    }

    pub fn load(&mut self) -> AlienResult<()> {
        let elf_binary = self.data;
        const ELF_MAGIC: [u8; 4] = [0x7f, b'E', b'L', b'F'];
        if elf_binary[0..4] != ELF_MAGIC {
            return Err(AlienError::EINVAL);
        }
        debug!("Domain address:{:p}", elf_binary.as_ptr());
        let elf = ElfFile::new(elf_binary).unwrap();
        debug!("Domain type:{:?}", elf.header.pt2.type_().as_type());
        let end_paddr = elf
            .program_iter()
            .filter(|ph| ph.get_type() == Ok(Type::Load))
            .last()
            .map(|x| x.virtual_addr() as usize + x.mem_size() as usize)
            .unwrap();
        let end_paddr = VirtAddr::from(end_paddr).align_up(FRAME_SIZE);
        // alloc free page to map elf
        let region_start = alloc_free_region(end_paddr.as_usize()).unwrap();
        trace!(
            "region range:{:#x}-{:#x}",
            region_start,
            region_start + end_paddr.as_usize()
        );
        self.phy_start = region_start;
        self.load_program(&elf)?;
        self.relocate_dyn(&elf)?;
        let entry = elf.header.pt2.entry_point() as usize + region_start;
        debug!("entry: {:#x}", entry);
        self.entry = entry;
        Ok(())
    }

    pub fn reload(&self) -> AlienResult<()> {
        info!("reload domain");
        let elf_binary = self.data;
        const ELF_MAGIC: [u8; 4] = [0x7f, b'E', b'L', b'F'];
        if elf_binary[0..4] != ELF_MAGIC {
            return Err(AlienError::EINVAL);
        }
        let elf = ElfFile::new(elf_binary).unwrap();
        self.reload_program(&elf)?;
        self.relocate_dyn(&elf)?;
        info!("reload domain done");
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
