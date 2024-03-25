use crate::vfs_shim;
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use basic::vm::{MappingFlags, PagingIf, PhysAddr, VirtAddr};
use config::{
    ELF_BASE_RELOCATE, FRAME_BITS, FRAME_SIZE, TRAMPOLINE, TRAP_CONTEXT_BASE, USER_STACK_SIZE,
};
use constants::{AlienError, AlienResult};
use core::cmp::min;
use core::mem::forget;
use ptable::*;
use xmas_elf::program::{SegmentData, Type};
use xmas_elf::ElfFile;

#[derive(Debug)]
pub struct VmmPageAllocator;

impl PagingIf for VmmPageAllocator {
    fn alloc_frame() -> Option<PhysAddr> {
        let frame = basic::frame::FrameTracker::new(1);
        let start = frame.start();
        forget(frame);
        Some(PhysAddr::from(start as usize))
    }

    fn dealloc_frame(paddr: PhysAddr) {
        let frame = basic::frame::FrameTracker::from_raw(paddr.as_usize(), 1);
        drop(frame);
    }

    fn phys_to_virt(paddr: PhysAddr) -> VirtAddr {
        VirtAddr::from(paddr.as_usize())
    }
}

#[derive(Debug)]
pub struct FrameTracker {
    // start page
    start: usize,
    // page count
    size: usize,
    // should be deallocated
    dealloc: bool,
}

impl FrameTracker {
    pub fn new(start_page: usize, pages: usize, dealloc: bool) -> Self {
        Self {
            start: start_page,
            size: pages,
            dealloc,
        }
    }

    pub fn from_addr(start: usize, pages: usize, dealloc: bool) -> Self {
        Self {
            start: start >> FRAME_BITS,
            size: pages,
            dealloc,
        }
    }
}

impl PhyPageMeta for FrameTracker {
    fn start_addr(&self) -> usize {
        self.start << FRAME_BITS
    }
    fn size(&self) -> usize {
        self.size * FRAME_SIZE
    }
}

impl Drop for FrameTracker {
    fn drop(&mut self) {
        if self.dealloc {
            trace!("drop frame tracker: {:#x?}", self);
            VmmPageAllocator::dealloc_frame(PhysAddr::from(self.start_addr()));
        }
    }
}

pub struct ELFInfo {
    pub address_space: VmSpace<VmmPageAllocator>,
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
            let frame = VmmPageAllocator::alloc_frame().unwrap();
            phy_frames.push(PhyFrame::new(Box::new(FrameTracker::from_addr(
                frame.as_usize(),
                1,
                true,
            ))));
        }

        let mut page_offset = section.start_vaddr & (FRAME_SIZE - 1);
        let mut count = 0;
        phy_frames.iter().for_each(|phy| {
            let size = FRAME_SIZE;
            let min = min(size - page_offset, data.len());
            let dst = (phy.start_addr() + page_offset) as *mut u8;
            unsafe {
                core::ptr::copy(data.as_ptr(), dst, min);
            }
            data = &data[min..];
            count += min;
            page_offset = 0;
        });
        assert_eq!(count, section.file_size);

        let area = VmArea::new(
            vaddr.as_usize()..end_vaddr.as_usize(),
            section.permission,
            phy_frames,
        );
        address_space.map(VmAreaType::VmArea(area)).unwrap();
    }

    Ok(break_addr)
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
        assert!(path.starts_with("/lib/ld-musl-riscv64-sf.so.1"));
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
        .unwrap_or(0);
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
    let mut user_stack_phy_frames = vec![];
    for _ in 0..USER_STACK_SIZE / FRAME_SIZE {
        let frame = VmmPageAllocator::alloc_frame().unwrap();
        user_stack_phy_frames.push(PhyFrame::new(Box::new(FrameTracker::from_addr(
            frame.as_usize(),
            1,
            true,
        ))));
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

    let trap_context_phy = VmmPageAllocator::alloc_frame().unwrap();
    let trap_context_area = VmArea::new(
        TRAP_CONTEXT_BASE..(TRAP_CONTEXT_BASE + FRAME_SIZE),
        MappingFlags::USER | MappingFlags::READ | MappingFlags::WRITE,
        vec![PhyFrame::new(Box::new(FrameTracker::from_addr(
            trap_context_phy.as_usize(),
            1,
            true,
        )))],
    );
    address_space
        .map(VmAreaType::VmArea(trap_context_area))
        .unwrap();

    // how to solve trampoline

    let trampoline_phy_addr = basic::trampoline_addr();
    let trampoline_area = VmArea::new(
        TRAMPOLINE..(TRAMPOLINE + FRAME_SIZE),
        MappingFlags::READ | MappingFlags::EXECUTE,
        vec![PhyFrame::new(Box::new(FrameTracker::from_addr(
            trampoline_phy_addr,
            1,
            false,
        )))],
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

    Ok(ELFInfo {
        address_space,
        entry: elf.header.pt2.entry_point() as usize + bias,
        stack_top: uer_stack_top,
        heap_bottom,
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
    let trampoline_phy_addr = basic::trampoline_addr();
    vm_space.area_iter().for_each(|ty| match ty {
        VmAreaType::VmArea(area) => {
            let size = area.size();
            let start = area.start();
            info!("<clone_vm_space> start: {:#x}, size: {:#x}", start, size);
            if start == trampoline_phy_addr {
                let trampoline_area = VmArea::new(
                    TRAMPOLINE..(TRAMPOLINE + FRAME_SIZE),
                    MappingFlags::READ | MappingFlags::EXECUTE,
                    vec![PhyFrame::new(Box::new(FrameTracker::from_addr(
                        trampoline_phy_addr,
                        1,
                        false,
                    )))],
                );
                space.map(VmAreaType::VmArea(trampoline_area)).unwrap();
            } else {
                let mut phy_frames = vec![];
                for _ in 0..size / FRAME_SIZE {
                    let frame = VmmPageAllocator::alloc_frame().unwrap();
                    phy_frames.push(PhyFrame::new(Box::new(FrameTracker::from_addr(
                        frame.as_usize(),
                        1,
                        true,
                    ))));
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
    let trap_context_phy = VmmPageAllocator::alloc_frame().unwrap();
    let trap_context_area = VmArea::new(
        address..(address + FRAME_SIZE),
        MappingFlags::USER | MappingFlags::READ | MappingFlags::WRITE,
        vec![PhyFrame::new(Box::new(FrameTracker::from_addr(
            trap_context_phy.as_usize(),
            1,
            true,
        )))],
    );
    space.map(VmAreaType::VmArea(trap_context_area)).unwrap();
}
