#![no_std]
extern crate alloc;

use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;
use config::FRAME_SIZE;
use core::mem::forget;
use domain_helper::{alloc_domain_id, DomainSyscall, SharedHeapAllocator};
use interface::{BlkDevice, Fs};
use libsyscall::Syscall;
use log::{info, trace, warn};
use mem::{alloc_free_region, map_region_to_kernel, query_kernel_space, MappingFlags, VirtAddr};
use proxy::{BlkDomainProxy, FsDomainProxy};
use rref::SharedHeap;
use xmas_elf::program::Type;
use xmas_elf::sections::{Rela, SectionData};
use xmas_elf::symbol_table::{DynEntry64, Entry};
use xmas_elf::{ElfFile, P64};

// pub struct DomainLoader;

static BLK_DOMAIN: &'static [u8] = include_bytes!("../../../build/blk_domain.bin");
static FATFS_DOMAIN: &'static [u8] = include_bytes!("../../../build/fatfs_domain.bin");

pub fn test_domain() {
    warn!("Load blk domain, size: {}KB", BLK_DOMAIN.len() / 1024);
    let entry = test_domain_loader(BLK_DOMAIN).unwrap();
    warn!("Load blk domain done, entry: {:#x}", entry);
    let dev = blk_domain(entry);

    warn!("Load fatfs domain, size: {}KB", FATFS_DOMAIN.len() / 1024);
    let entry = test_domain_loader(FATFS_DOMAIN).unwrap();
    warn!("Load fatfs domain done, entry: {:#x}", entry);
    let fs = fatfs_domain(entry, dev);
    forget(fs);
    // fs.drop_self();
}

fn fatfs_domain(entry: usize, dev: Box<dyn BlkDevice>) -> Box<dyn Fs> {
    type F = fn(
        Box<dyn Syscall>,
        domain_id: u64,
        Box<dyn SharedHeap>,
        Box<dyn BlkDevice>,
    ) -> Box<dyn Fs>;
    let f: F = unsafe { core::mem::transmute::<*const (), F>(entry as *const ()) };
    let id = alloc_domain_id();
    let fatfs = f(
        Box::new(DomainSyscall),
        id,
        Box::new(SharedHeapAllocator),
        dev,
    );
    // fatfs.drop_self();
    Box::new(FsDomainProxy::new(id, fatfs))
}

fn blk_domain(entry: usize) -> Box<dyn BlkDevice> {
    type F = fn(Box<dyn Syscall>, domain_id: u64, Box<dyn SharedHeap>, usize) -> Box<dyn BlkDevice>;
    let f: F = unsafe { core::mem::transmute::<*const (), F>(entry as *const ()) };
    let id = alloc_domain_id();
    let dev = f(
        Box::new(DomainSyscall),
        id,
        Box::new(SharedHeapAllocator),
        0x10008000,
    );
    info!(
        "dev capacity: {:?}MB",
        dev.get_capacity().unwrap() / 1024 / 1024
    );
    // dev.drop_self();
    Box::new(BlkDomainProxy::new(id, dev))
    // dev
}

pub fn test_domain_loader(elf: &[u8]) -> Result<usize, &'static str> {
    const ELF_MAGIC: [u8; 4] = [0x7f, b'E', b'L', b'F'];
    if elf[0..4] != ELF_MAGIC {
        return Err("ELFError::InvalidELF");
    }
    let elf = xmas_elf::ElfFile::new(elf).unwrap();
    info!("Domain type:{:?}", elf.header.pt2.type_().as_type());
    let end_paddr = elf
        .program_iter()
        .filter(|ph| ph.get_type() == Ok(Type::Load))
        .last()
        .map(|x| x.virtual_addr() as usize + x.mem_size() as usize)
        .unwrap();
    let end_paddr = VirtAddr::from(end_paddr).align_up(FRAME_SIZE);

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
            info!(
                "map range: [{:#x}-{:#x}], memsize:{}, perm:{:?}",
                vaddr,
                vaddr + len,
                ph.mem_size(),
                permission
            );
            map_region_to_kernel(vaddr, len, permission).unwrap();
            let data = &elf.input[ph.offset() as usize..(ph.offset() + ph.file_size()) as usize];
            let mut page_offset = start_vaddr & (FRAME_SIZE - 1);
            let mut count = 0;
            while count < data.len() {
                let paddr = query_kernel_space(vaddr).unwrap();
                let len = core::cmp::min(FRAME_SIZE - page_offset, data.len() - count);
                let dst_buf =
                    unsafe { core::slice::from_raw_parts_mut(paddr as *mut u8, FRAME_SIZE) };
                dst_buf[page_offset..page_offset + len].copy_from_slice(&data[count..count + len]);
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
    // drop dev will cause fault
    Ok(entry)
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

// const R_RISCV_32: u32 = 1;
const R_RISCV_64: u32 = 2;
const R_RISCV_RELATIVE: u32 = 3;
// const R_RISCV_COPY: u32 = 4;
const R_RISCV_JUMP_SLOT: u32 = 5;
// const REL_GOT: u32 = 6;
// const REL_PLT: u32 = 7;
// const REL_RELATIVE: u32 = 8;

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
