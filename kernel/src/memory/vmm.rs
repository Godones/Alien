use alloc::sync::Arc;
use core::cmp::min;
use core::fmt::{Debug, Formatter};
use core::intrinsics::forget;

use lazy_static::lazy_static;
use page_table::addr::{align_up_4k, PhysAddr, VirtAddr};
use page_table::pte::MappingFlags;
use page_table::table::{PagingIf, Sv39PageTable};
use spin::RwLock;
use xmas_elf::program;

use crate::config::{FRAME_BITS, FRAME_SIZE, MMIO, TRAMPOLINE, TRAP_CONTEXT_BASE, USER_STACK_SIZE};
use crate::memory::frame::{addr_to_frame, frame_alloc};
use crate::memory::{frame_alloc_contiguous, FRAME_REF_MANAGER};

lazy_static! {
    pub static ref KERNEL_SPACE: Arc<RwLock<Sv39PageTable<PageAllocator>>> = Arc::new(RwLock::new(
        Sv39PageTable::<PageAllocator>::try_new().unwrap()
    ));
}

#[allow(unused)]
extern "C" {
    fn stext();
    fn srodata();
    fn sdata();
    fn sbss();
    fn ekernel();
    fn strampoline();
}

pub fn kernel_info(memory_end: usize) {
    println!(
        "kernel text:   {:#x}-{:#x}",
        stext as usize, srodata as usize
    );
    println!(
        "kernel rodata: {:#x}-{:#x}",
        srodata as usize, sdata as usize
    );
    println!("kernel data:   {:#x}-{:#x}", sdata as usize, sbss as usize);
    println!(
        "kernel bss:    {:#x}-{:#x}",
        sbss as usize, ekernel as usize
    );
    println!("kernel heap:   {:#x}-{:#x}", ekernel as usize, memory_end);
}

/// 建立内核页表
pub fn build_kernel_address_space(memory_end: usize) {
    let mut kernel_space = KERNEL_SPACE.write();
    kernel_space
        .map_region(
            VirtAddr::from(stext as usize),
            PhysAddr::from(stext as usize),
            srodata as usize - stext as usize,
            "RXVAD".into(),
            true,
        )
        .unwrap();
    kernel_space
        .map_region(
            VirtAddr::from(srodata as usize),
            PhysAddr::from(srodata as usize),
            sdata as usize - srodata as usize,
            "RVAD".into(),
            true,
        )
        .unwrap();
    kernel_space
        .map_region(
            VirtAddr::from(sdata as usize),
            PhysAddr::from(sdata as usize),
            sbss as usize - sdata as usize,
            "RWVAD".into(),
            true,
        )
        .unwrap();
    kernel_space
        .map_region(
            VirtAddr::from(sbss as usize),
            PhysAddr::from(sbss as usize),
            ekernel as usize - sbss as usize,
            "RWVAD".into(),
            true,
        )
        .unwrap();
    kernel_space
        .map_region(
            VirtAddr::from(ekernel as usize),
            PhysAddr::from(ekernel as usize),
            memory_end - ekernel as usize,
            "RWVAD".into(),
            true,
        )
        .unwrap();
    kernel_space
        .map_region(
            VirtAddr::from(TRAMPOLINE),
            PhysAddr::from(strampoline as usize),
            FRAME_SIZE,
            "RXVAD".into(),
            true,
        )
        .unwrap();
    for pair in MMIO {
        kernel_space
            .map_region(
                VirtAddr::from(pair.0),
                PhysAddr::from(pair.0),
                pair.1,
                "RWVAD".into(),
                true,
            )
            .unwrap();
    }
}

pub struct ELFInfo {
    pub address_space: Sv39PageTable<PageAllocator>,
    pub entry: usize,
    pub stack_top: usize,
    pub heap_bottom: usize,
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

#[derive(Debug)]
pub enum ELFError {
    NotELF,
    FileBreak,
    NotSupported,
    NoLoadableSegment,
    NoStackSegment,
    NoEntrySegment,
}

pub fn build_clone_address_space(
    p_table: &mut Sv39PageTable<PageAllocator>,
) -> Sv39PageTable<PageAllocator> {
    let mut address_space = Sv39PageTable::<PageAllocator>::try_new().unwrap();
    p_table
        .get_record()
        .into_iter()
        .for_each(|(v_addr, target)| {
            let (phy, mut flag, page_size) = p_table.query(v_addr).unwrap();
            if v_addr.as_usize() == TRAP_CONTEXT_BASE {
                // for Trap_context, we remap it
                assert_eq!(usize::from(page_size), TRAMPOLINE - TRAP_CONTEXT_BASE);
                let dst = address_space
                    .map_no_target(v_addr, page_size, flag, false)
                    .unwrap();
                // copy data
                let src_ptr = phy.as_usize() as *const u8;
                let dst_ptr = dst.as_usize() as *mut u8;
                unsafe {
                    core::ptr::copy(src_ptr, dst_ptr, usize::from(page_size));
                }
            } else {
                // cow
                // checkout whether pte flags has `W` flag
                let new_flag = if flag.contains(MappingFlags::W) {
                    flag -= MappingFlags::W;
                    flag |= MappingFlags::RSD; // we use the RSD flag to indicate that this page is a cow page
                                               // update parent's flag and clear dirty
                    p_table.modify_pte_flags(v_addr, flag, false).unwrap();
                    flag
                } else {
                    flag
                };
                address_space.map(v_addr, phy, page_size, new_flag).unwrap();
                // add ref for alloc page
                let page_info = address_space.alloc_pages_info_mut();
                if target.is_none() {
                    for i in 0..usize::from(page_size) / FRAME_SIZE {
                        page_info.push(phy + FRAME_SIZE * i);
                        let page_number = (phy + FRAME_SIZE * i).as_usize() >> FRAME_BITS;
                        FRAME_REF_MANAGER.lock().get_ref(page_number);
                        FRAME_REF_MANAGER.lock().add_ref(page_number);
                    }
                }
            }
        });
    address_space
}

pub fn build_elf_address_space(elf: &[u8]) -> Result<ELFInfo, ELFError> {
    let mut address_space = Sv39PageTable::<PageAllocator>::try_new().unwrap();
    const ELF_MAGIC: [u8; 4] = [0x7f, b'E', b'L', b'F'];
    if elf[0..4] != ELF_MAGIC {
        return Err(ELFError::NotELF);
    }
    let elf = xmas_elf::ElfFile::new(elf).map_err(|_| ELFError::NotELF)?;
    let mut break_addr = 0usize;
    let phs = elf.header.pt2.ph_count();
    for i in 0..phs {
        let ph = elf.program_header(i).map_err(|_| ELFError::FileBreak)?;
        let p_type = ph.get_type().map_err(|_| ELFError::NotSupported)?;
        if p_type == program::Type::Load {
            let start_addr = ph.virtual_addr() as usize;
            let end_addr = start_addr + ph.mem_size() as usize;
            // 记录程序地址空间的最大地址
            break_addr = end_addr;
            let mut permission: MappingFlags = "UVAD".into();
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
            trace!(
                "load segment: {:#x} - {:#x}, permission: {:?}",
                start_addr,
                end_addr,
                permission
            );
            let mut data =
                &elf.input[ph.offset() as usize..(ph.offset() + ph.file_size()) as usize];
            let map_info = address_space
                .map_region_no_target(
                    VirtAddr::from(start_addr).align_down_4k(),
                    align_up_4k(end_addr - start_addr),
                    permission,
                    true,
                    false,
                )
                .unwrap();
            // copy data
            map_info
                .into_iter()
                .for_each(|(vir, phy, page_size)| unsafe {
                    trace!("{:#x} {:#x} {:#x?}", vir, phy, page_size);
                    let size: usize = page_size.into();
                    let min = min(size, data.len());
                    let dst = phy.as_usize() as *mut u8;
                    core::ptr::copy(data.as_ptr(), dst, min);
                    data = &data[min..];
                })
        }
    }
    // 地址向上取整对齐4
    let ceil_addr = align_up_4k(break_addr);
    // 留出一个用户栈的位置+隔离页
    let top = ceil_addr + USER_STACK_SIZE + FRAME_SIZE; // 8k +4k

    // map user stack
    address_space
        .map_region_no_target(
            VirtAddr::from(top - USER_STACK_SIZE),
            USER_STACK_SIZE,
            "RWUVAD".into(),
            true,
            false,
        )
        .unwrap();

    // todo!(heap)
    let heap_bottom = top; // align to 4k
    address_space
        .map_region_no_target(
            VirtAddr::from(TRAP_CONTEXT_BASE),
            TRAMPOLINE - TRAP_CONTEXT_BASE,
            "RWVAD".into(),
            true,
            false,
        )
        .unwrap();
    address_space
        .map_region(
            VirtAddr::from(TRAMPOLINE),
            PhysAddr::from(strampoline as usize),
            FRAME_SIZE,
            "RXVAD".into(),
            true,
        )
        .unwrap();

    Ok(ELFInfo {
        address_space,
        entry: elf.header.pt2.entry_point() as usize,
        stack_top: top,
        heap_bottom,
    })
}

pub struct PageAllocator;

impl PagingIf for PageAllocator {
    fn alloc_frame() -> Option<PhysAddr> {
        frame_alloc().map(|frame| {
            let start = frame.start();
            trace!("PageAllocator alloc frame{:?} start:{:#x}", frame, start);
            forget(frame);
            PhysAddr::from(start)
        })
    }

    fn dealloc_frame(paddr: PhysAddr) {
        let frame = addr_to_frame(paddr.as_usize());
        trace!("PageAllocator dealloc frame {:?}", frame);
    }

    fn phys_to_virt(paddr: PhysAddr) -> VirtAddr {
        VirtAddr::from(paddr.as_usize())
    }

    fn alloc_contiguous_frames(size: usize) -> Option<PhysAddr> {
        let ptr = frame_alloc_contiguous(size);
        if ptr.is_null() {
            return None;
        }
        Some(PhysAddr::from(ptr as usize))
    }
}
