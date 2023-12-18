#![no_std]
extern crate alloc;

use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;
use core::ops::Range;
use log::{info, trace, warn};
use xmas_elf::ElfFile;
use xmas_elf::program::{Type};
use xmas_elf::sections::SectionData;
use xmas_elf::symbol_table::Entry;
use config::FRAME_SIZE;
use domain_helper::DomainSyscall;
use interface::BlkDevice;
use libsyscall::Syscall;
use mem::{alloc_free_region, map_region_to_kernel, MappingFlags, query_kernel_space, VirtAddr};

pub struct DomainLoader;


impl DomainLoader{
    pub fn load(domain_name: &str, binary: &[u8], args: &[&str]) -> Result<(), ()>{
        Ok(())
    }
}

static BLK_DOMAIN:&'static [u8] = include_bytes!("../../../build/blk_domain.bin");

#[no_mangle]
pub fn test_domain_loader() -> Result<(),&'static str> {
    let elf  = BLK_DOMAIN;
    const ELF_MAGIC: [u8; 4] = [0x7f, b'E', b'L', b'F'];
    if elf[0..4] != ELF_MAGIC {
        return Err("ELFError::InvalidELF");
    }
    let elf = xmas_elf::ElfFile::new(elf).unwrap();
    info!("type:{:?}", elf.header.pt2.type_().as_type());

    let end_paddr = elf.program_iter()
        .filter(|ph| ph.get_type() == Ok(Type::Load))
        .last().map(|x| x.virtual_addr() as usize + x.mem_size() as usize).unwrap();
    let end_paddr = VirtAddr::from(end_paddr).align_up(FRAME_SIZE);

    let region_start = alloc_free_region(end_paddr.as_usize()).unwrap();

    info!("region range:{:#x}-{:#x}", region_start, region_start + end_paddr.as_usize());
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
            info!("map range: [{:#x}-{:#x}], memsize:{}, perm:{:?}" ,vaddr, vaddr + len,ph.mem_size(),permission);
            map_region_to_kernel(vaddr, len, permission).unwrap();
            let data =
                &elf.input[ph.offset() as usize..(ph.offset() + ph.file_size()) as usize];
            let mut page_offset = start_vaddr & (FRAME_SIZE-1);
            let mut count = 0;
            while count < data.len() {
                let paddr = query_kernel_space(vaddr).unwrap();
                let len = core::cmp::min(FRAME_SIZE - page_offset, data.len() - count);
                let dst_buf = unsafe {
                    core::slice::from_raw_parts_mut(paddr as *mut u8, FRAME_SIZE)
                };
                dst_buf[page_offset..page_offset + len].copy_from_slice(&data[count..count + len]);
                info!("copy data to {:#x}-{:#x}", paddr, paddr + len);
                vaddr += len;
                page_offset = 0;
                count += len;
            }
            assert_eq!(count, data.len());
        });

    if let Ok(res) = relocate_dyn(&elf,region_start){
        res.into_iter().for_each(|kv| {
            info!("relocate: {:#x} -> {:#x}", kv.0, kv.1);
            let addr = query_kernel_space(kv.0).unwrap();
            unsafe { (addr as *mut usize).write(kv.1) }
        });
    }
    if let Ok(res) = relocate_plt(&elf,region_start){
        res.into_iter().for_each(|kv| {
            info!("relocate: {:#x} -> {:#x}", kv.0, kv.1);
            let addr = query_kernel_space(kv.0).unwrap();
            unsafe { (addr as *mut usize).write(kv.1) }
        });
    }
    let entry = elf.header.pt2.entry_point() as usize + region_start;
    info!("entry: {:#x}", entry);
    test_dump(2);
    type F =  fn(Box<dyn Syscall>, domain_id: u64) -> Box<dyn BlkDevice>;
    let f: F = unsafe { core::mem::transmute(entry) };
    let dev = f(Box::new(DomainSyscall), 0);
    test_dump(1);
    info!("dev");
    // info!("dev len: {:?}",dev.get_capacity());
    Ok(())
}

#[no_mangle]
fn test_dump(x:usize){
    info!("dump it");
    for i in 0..1000{
        let x = 10;
        if x %1000 == 0{
            info!("dump it");
        }
    }
    info!("x {}",x/10);
}
fn relocate_dyn(elf:&ElfFile,region_start:usize)->Result<Vec<(usize,usize)>,&'static str>{
    let mut res = vec![];
    let data = elf
        .find_section_by_name(".rela.dyn")
        .map(|h|{
            h.get_data(&elf).unwrap()
        }).ok_or("corrupted .rela.dyn")?;
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
    for entry in entries.iter() {
        match entry.get_type() {
            REL_GOT | REL_PLT | R_RISCV_64  => {
                let dynsym = &dynsym[entry.get_symbol_table_index() as usize];
                let symval = if dynsym.shndx() == 0 {
                    let name = dynsym.get_name(&elf)?;
                    panic!("need to find symbol: {:?}", name);
                } else {
                    region_start + dynsym.value() as usize
                };
                let value = symval + entry.get_addend() as usize;
                let addr = region_start + entry.get_offset() as usize;
                res.push((addr, value))
            }
            REL_RELATIVE | R_RISCV_RELATIVE  => {
                let value = region_start + entry.get_addend() as usize;
                let addr = region_start + entry.get_offset() as usize;
                res.push((addr, value))
            }
            t => unimplemented!("unknown type: {}", t),
        }
    }
    Ok(res)
}

fn relocate_plt(elf:&ElfFile,region_start:usize)->Result<Vec<(usize,usize)>,&'static str>{
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
                warn!("dynsym: {:?}", dynsym);
                let value = region_start + symval+ entry.get_addend() as usize;
                let addr = region_start + entry.get_offset() as usize;
                res.push((addr, value))
            }
            t => panic!("[kernel] unknown entry, type = {}", t),
        }
    }
    Ok(res)
}

const R_RISCV_32: u32 = 1;
const R_RISCV_64: u32 = 2;
const R_RISCV_RELATIVE: u32 = 3;
const R_RISCV_COPY: u32 = 4;
const R_RISCV_JUMP_SLOT: u32 = 5;
const REL_GOT: u32 = 6;
const REL_PLT: u32 = 7;
const REL_RELATIVE: u32 = 8;
