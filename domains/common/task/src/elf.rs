use alloc::{
    boxed::Box,
    string::{String, ToString},
    vec,
    vec::Vec,
};
use core::{
    cmp::min,
    fmt::Debug,
    ops::{Deref, DerefMut},
};

use basic::{config::*, vm::frame::FrameTracker, AlienError, AlienResult};
use memory_addr::{PhysAddr, VirtAddr};
use page_table::{MappingFlags, NotLeafPage, PagingIf, Rv64PTE};
use ptable::*;
use xmas_elf::{
    program::{SegmentData, Type},
    sections::SectionData,
    symbol_table::Entry,
    ElfFile,
};

use crate::vfs_shim;

#[derive(Debug)]
pub struct FrameTrackerWrapper(pub(crate) FrameTracker);
impl NotLeafPage<Rv64PTE> for FrameTrackerWrapper {
    fn phys_addr(&self) -> PhysAddr {
        self.0.start_phy_addr()
    }

    fn virt_addr(&self) -> VirtAddr {
        self.0.start_virt_addr()
    }

    fn zero(&self) {
        self.0.clear();
    }

    fn as_pte_slice<'a>(&self) -> &'a [Rv64PTE] {
        self.0.as_slice_with(0)
    }

    fn as_pte_mut_slice<'a>(&self) -> &'a mut [Rv64PTE] {
        self.0.as_mut_slice_with(0)
    }
}

impl Into<FrameTrackerWrapper> for FrameTracker {
    fn into(self) -> FrameTrackerWrapper {
        FrameTrackerWrapper(self)
    }
}

impl Deref for FrameTrackerWrapper {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for FrameTrackerWrapper {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl PhysPage for FrameTrackerWrapper {
    fn phys_addr(&self) -> PhysAddr {
        self.0.start_phy_addr()
    }

    fn as_bytes(&self) -> &[u8] {
        self.0.as_slice_with(0)
    }

    fn as_mut_bytes(&mut self) -> &mut [u8] {
        self.0.as_mut_slice_with(0)
    }
}

#[derive(Debug)]
pub struct VmmPageAllocator;

impl PagingIf<Rv64PTE> for VmmPageAllocator {
    fn alloc_frame() -> Option<Box<dyn NotLeafPage<Rv64PTE>>> {
        let frame = FrameTracker::new(1);
        Some(Box::new(FrameTrackerWrapper(frame)))
    }
}

pub struct ELFInfo {
    pub address_space: VmSpace<VmmPageAllocator>,
    pub entry: VirtAddr,
    pub stack_top: VirtAddr,
    pub heap_bottom: VirtAddr,
    pub ph_num: usize,
    pub ph_entry_size: usize,
    pub ph_drift: usize,
    pub tls: usize,
    pub bias: usize,
    pub name: String,
}

pub fn calculate_bias(elf: &ElfFile) -> AlienResult<usize> {
    let bias = match elf.header.pt2.type_().as_type() {
        // static
        xmas_elf::header::Type::Executable => 0,
        xmas_elf::header::Type::SharedObject => {
            match elf
                .program_iter()
                .filter(|ph| ph.get_type().unwrap() == Type::Interp)
                .count()
            {
                // It's a loader!
                0 => ELF_BASE_RELOCATE,
                // It's a dynamically linked ELF.
                1 => 0,
                // Emmm, It has multiple interpreters.
                _ => return Err(AlienError::ENOSYS),
            }
        }
        _ => return Err(AlienError::ENOSYS),
    };
    trace!("bias: {:#x}", bias);
    Ok(bias)
}

struct LoadInfo {
    start_vaddr: usize,
    end_vaddr: usize,
    permission: MappingFlags,
    offset: usize,
    file_size: usize,
}

fn collect_load_info(elf: &ElfFile, bias: usize) -> Vec<LoadInfo> {
    let mut info = vec![];
    elf.program_iter()
        .filter(|ph| ph.get_type() == Ok(Type::Load))
        .for_each(|ph| {
            let start_addr = ph.virtual_addr() as usize + bias;
            let end_addr = start_addr + ph.mem_size() as usize;
            let mut permission: MappingFlags = MappingFlags::USER;
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
            let load_info = LoadInfo {
                start_vaddr: start_addr,
                end_vaddr: end_addr,
                permission,
                offset: ph.offset() as usize,
                file_size: ph.file_size() as usize,
            };
            info.push(load_info);
        });
    info
}

pub fn load_to_vm_space(
    elf: &ElfFile,
    bias: usize,
    address_space: &mut VmSpace<VmmPageAllocator>,
) -> AlienResult<usize> {
    let mut break_addr = 0usize;
    let info = collect_load_info(elf, bias);

    for section in info {
        let vaddr = VirtAddr::from(section.start_vaddr).align_down_4k();
        let end_vaddr = VirtAddr::from(section.end_vaddr).align_up_4k();
        break_addr = section.end_vaddr;
        let len = end_vaddr.as_usize() - vaddr.as_usize();
        warn!(
            "load segment: {:#x} - {:#x} -> {:#x}-{:#x}, permission: {:?}",
            section.start_vaddr,
            section.end_vaddr,
            vaddr.as_usize(),
            end_vaddr.as_usize(),
            section.permission
        );
        let mut data = &elf.input[section.offset..(section.offset + section.file_size)];
        let mut phy_frames = vec![];
        for _ in 0..len / FRAME_SIZE {
            let frame = FrameTracker::new(1);
            phy_frames.push(Box::new(FrameTrackerWrapper(frame)));
        }

        let mut page_offset = section.start_vaddr & (FRAME_SIZE - 1);
        let mut count = 0;
        phy_frames.iter_mut().for_each(|phy_frame| {
            let size = FRAME_SIZE;
            let min = min(size - page_offset, data.len());
            phy_frame[page_offset..(page_offset + min)].copy_from_slice(&data[..min]);
            data = &data[min..];
            count += min;
            page_offset = 0;
        });
        assert_eq!(count, section.file_size);
        let phy_frames = phy_frames.into_iter().map(|x| x as _).collect();

        let area = VmArea::new(
            vaddr.as_usize()..end_vaddr.as_usize(),
            section.permission,
            phy_frames,
        );
        address_space.map(VmAreaType::VmArea(area)).unwrap();
    }

    Ok(break_addr)
}

fn relocate_dyn(elf: &ElfFile, bias: usize) -> AlienResult<Vec<(usize, usize)>> {
    let mut res = vec![];
    let data = elf
        .find_section_by_name(".rela.dyn")
        .unwrap()
        .get_data(elf)
        .unwrap();
    let entries = match data {
        SectionData::Rela64(entries) => entries,
        _ => return Err(AlienError::EINVAL),
    };
    let dynsym = match elf
        .find_section_by_name(".dynsym")
        .unwrap()
        .get_data(elf)
        .unwrap()
    {
        SectionData::DynSymbolTable64(dsym) => dsym,
        _ => return Err(AlienError::EINVAL),
    };
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
                    let name = dynsym.get_name(elf).map_err(|_| AlienError::EINVAL)?;
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
    Ok(res)
}

fn relocate_plt(elf: &ElfFile, bias: usize) -> AlienResult<Vec<(usize, usize)>> {
    let mut res = vec![];
    let data = elf
        .find_section_by_name(".rela.plt")
        .ok_or(AlienError::EINVAL)?
        .get_data(elf)
        .map_err(|_| AlienError::EINVAL)?;
    let entries = match data {
        SectionData::Rela64(entries) => entries,
        _ => return Err(AlienError::EINVAL),
    };
    let dynsym = match elf
        .find_section_by_name(".dynsym")
        .unwrap()
        .get_data(elf)
        .unwrap()
    {
        SectionData::DynSymbolTable64(dsym) => dsym,
        _ => return Err(AlienError::EINVAL),
    };
    for entry in entries.iter() {
        match entry.get_type() {
            5 => {
                let dynsym = &dynsym[entry.get_symbol_table_index() as usize];
                let symval = if dynsym.shndx() == 0 {
                    let name = dynsym.get_name(elf).map_err(|_| AlienError::EINVAL)?;
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

pub fn build_vm_space(elf: &[u8], args: &mut Vec<String>, name: &str) -> AlienResult<ELFInfo> {
    let elf = xmas_elf::ElfFile::new(elf).map_err(|_| AlienError::EINVAL)?;
    // if the elf file is a shared object, we should load the interpreter first
    if let Some(inter) = elf
        .program_iter()
        .find(|ph| ph.get_type().unwrap() == Type::Interp)
    {
        let data = match inter.get_data(&elf).unwrap() {
            SegmentData::Undefined(data) => data,
            _ => return Err(AlienError::EINVAL),
        };
        let path = core::str::from_utf8(data).unwrap();
        assert!(path.starts_with("/lib/ld-musl-riscv64"));
        let mut new_args = vec!["/libc.so\0".to_string()];
        new_args.extend(args.clone());
        *args = new_args;
        // load interpreter
        let mut data = vec![];
        info!("load interpreter: {}, new_args:{:?}", path, args);
        if vfs_shim::read_all("libc.so", &mut data) {
            return build_vm_space(&data, args, "libc.so");
        } else {
            panic!(
                "[build_vm_space] Found interpreter path: {}， but read error",
                path
            );
        }
    };

    let bias = calculate_bias(&elf)?;

    let tls = elf
        .program_iter()
        .find(|x| x.get_type().unwrap() == Type::Tls)
        .map(|ph| ph.virtual_addr())
        .unwrap_or(0 + bias as u64);
    info!("ELF tls: {:#x}", tls);

    let mut address_space = VmSpace::new();
    let break_addr = load_to_vm_space(&elf, bias, &mut address_space)?;

    // user stack
    let ceil_addr = PhysAddr::from(break_addr + FRAME_SIZE)
        .align_up_4k()
        .as_usize();

    let user_stack_low = ceil_addr + FRAME_SIZE;
    let uer_stack_top = user_stack_low + USER_STACK_SIZE;
    warn!("user stack: {:#x} - {:#x}", user_stack_low, uer_stack_top);

    let mut user_stack_phy_frames: Vec<Box<dyn PhysPage>> = vec![];
    for _ in 0..USER_STACK_SIZE / FRAME_SIZE {
        let frame = FrameTracker::new(1);
        user_stack_phy_frames.push(Box::new(FrameTrackerWrapper(frame)));
    }
    let user_stack_area = VmArea::new(
        user_stack_low..uer_stack_top,
        MappingFlags::USER | MappingFlags::READ | MappingFlags::WRITE,
        user_stack_phy_frames,
    );
    address_space
        .map(VmAreaType::VmArea(user_stack_area))
        .unwrap();

    let heap_bottom = uer_stack_top;

    let trap_context_frame = FrameTracker::new(1);
    let trap_context_area = VmArea::new(
        TRAP_CONTEXT_BASE..(TRAP_CONTEXT_BASE + FRAME_SIZE),
        MappingFlags::USER | MappingFlags::READ | MappingFlags::WRITE,
        vec![Box::new(FrameTrackerWrapper(trap_context_frame))],
    );
    address_space
        .map(VmAreaType::VmArea(trap_context_area))
        .unwrap();

    // todo!(how to solve trampoline)
    let trampoline_frame = FrameTracker::create_trampoline();

    let trampoline_area = VmArea::new(
        TRAMPOLINE..(TRAMPOLINE + FRAME_SIZE),
        MappingFlags::READ | MappingFlags::EXECUTE,
        vec![Box::new(FrameTrackerWrapper(trampoline_frame))],
    );
    address_space
        .map(VmAreaType::VmArea(trampoline_area))
        .unwrap();

    let res = if let Some(phdr) = elf
        .program_iter()
        .find(|ph| ph.get_type() == Ok(Type::Phdr))
    {
        // if phdr exists in program header, use it
        Ok(phdr.virtual_addr())
    } else if let Some(elf_addr) = elf
        .program_iter()
        .find(|ph| ph.get_type() == Ok(Type::Load) && ph.offset() == 0)
    {
        // otherwise, check if elf is loaded from the beginning, then phdr can be inferred.
        // Ok(elf_addr.virtual_addr())
        Ok(elf_addr.virtual_addr() + elf.header.pt2.ph_offset())
    } else {
        warn!("elf: no phdr found, tls might not work");
        Err(AlienError::EINVAL)
    }
    .unwrap_or(0);
    warn!(
        "entry: {:#x}, phdr:{:#x}",
        elf.header.pt2.entry_point() + bias as u64,
        res + bias as u64
    );
    // todo!(relocate)
    if bias != 0 {
        // if the elf file is a shared object, we should relocate it
        if let Ok(kvs) = relocate_dyn(&elf, bias) {
            kvs.into_iter().for_each(|kv| {
                debug!("relocate: {:#x} -> {:#x}", kv.0, kv.1);
                address_space
                    .write_val(VirtAddr::from(kv.0), &kv.1)
                    .unwrap()
            });
            info!("relocate dynamic section success")
        }
        if let Ok(kvs) = relocate_plt(&elf, bias) {
            kvs.into_iter().for_each(|kv| {
                debug!("relocate: {:#x} -> {:#x}", kv.0, kv.1);
                address_space
                    .write_val(VirtAddr::from(kv.0), &kv.1)
                    .unwrap()
            });
            info!("relocate plt section success");
        }
    }

    Ok(ELFInfo {
        address_space,
        entry: VirtAddr::from(elf.header.pt2.entry_point() as usize + bias),
        stack_top: VirtAddr::from(uer_stack_top),
        heap_bottom: VirtAddr::from(heap_bottom),
        ph_num: elf.header.pt2.ph_count() as usize,
        ph_entry_size: elf.header.pt2.ph_entry_size() as usize,
        ph_drift: res as usize + bias,
        tls: tls as usize,
        bias,
        name: name.to_string(),
    })
}

pub fn clone_vm_space(vm_space: &VmSpace<VmmPageAllocator>) -> VmSpace<VmmPageAllocator> {
    let mut space = VmSpace::new();
    let trampoline_frame = FrameTracker::create_trampoline();
    let trampoline_frame_virt_addr = trampoline_frame.start_virt_addr().as_usize();
    vm_space.area_iter().for_each(|ty| match ty {
        VmAreaType::VmArea(area) => {
            let size = area.size();
            let start = area.start();
            info!("<clone_vm_space> start: {:#x}, size: {:#x}", start, size);
            if start == trampoline_frame_virt_addr {
                let trampoline_frame = FrameTracker::create_trampoline();
                let trampoline_area = VmArea::new(
                    TRAMPOLINE..(TRAMPOLINE + FRAME_SIZE),
                    MappingFlags::READ | MappingFlags::EXECUTE,
                    vec![Box::new(FrameTrackerWrapper(trampoline_frame))],
                );
                space.map(VmAreaType::VmArea(trampoline_area)).unwrap();
            } else {
                let mut phy_frames: Vec<Box<dyn PhysPage>> = vec![];
                for _ in 0..size / FRAME_SIZE {
                    let frame = FrameTracker::new(1);
                    phy_frames.push(Box::new(FrameTrackerWrapper(frame)));
                }
                let new_area = area.clone_with(phy_frames);
                space.map(VmAreaType::VmArea(new_area)).unwrap();
            }
        }
        VmAreaType::VmAreaEqual(area_eq) => {
            let new_area_eq = area_eq.clone();
            space.map(VmAreaType::VmAreaEqual(new_area_eq)).unwrap();
        }
    });
    space
}

pub fn extend_thread_vm_space(space: &mut VmSpace<VmmPageAllocator>, thread_num: usize) {
    assert!(thread_num > 0);
    let address = TRAP_CONTEXT_BASE - FRAME_SIZE * thread_num;
    let trap_context_frame = FrameTracker::new(1);
    let trap_context_area = VmArea::new(
        address..(address + FRAME_SIZE),
        MappingFlags::USER | MappingFlags::READ | MappingFlags::WRITE,
        vec![Box::new(FrameTrackerWrapper(trap_context_frame))],
    );
    space.map(VmAreaType::VmArea(trap_context_area)).unwrap();
}
