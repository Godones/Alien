use alloc::sync::Arc;
use core::cmp::min;
use core::fmt::{Debug, Formatter};
use core::intrinsics::forget;

use lazy_static::lazy_static;
use page_table::addr::{align_up_4k, PhysAddr, VirtAddr};
use page_table::pte::MappingFlags;
use page_table::table::{PagingIf, Sv39PageTable};
use xmas_elf::program;
use xmas_elf::program::Type;

use kernel_sync::RwLock;

use crate::config::{FRAME_BITS, FRAME_SIZE, MMIO, TRAMPOLINE, TRAP_CONTEXT_BASE, USER_STACK_SIZE};
use crate::memory::{frame_alloc_contiguous, FRAME_REF_MANAGER};
use crate::memory::frame::{addr_to_frame, frame_alloc};

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
    pub ph_num: usize,
    pub ph_entry_size: usize,
    pub ph_drift: usize,
}

#[derive(Debug)]
pub struct UserStack {
    pub virt_stack_top: usize,
    pub stack_top: usize,
    pub stack_bottom: usize,
}

impl UserStack {
    pub fn new(phy_stack_top: usize, virt_stack_top: usize) -> Self {
        Self {
            virt_stack_top,
            stack_top: phy_stack_top,
            stack_bottom: phy_stack_top - FRAME_SIZE,
        }
    }

    pub fn get_stack_top(&self) -> usize {
        self.stack_top
    }

    pub fn push(&mut self, data: usize) -> Result<usize, &'static str> {
        if self.stack_top - 8 < self.stack_bottom {
            return Err("Stack Overflow");
        }
        unsafe {
            self.stack_top -= 8;
            *(self.stack_top as *mut usize) = data;
        }
        trace!(
            "stack top: {:#x}, data:{:#x?}",
            self.virt_stack_top - (FRAME_SIZE - (self.stack_top - self.stack_bottom)),
            data
        );
        Ok(self.virt_stack_top - (FRAME_SIZE - (self.stack_top - self.stack_bottom)))
    }

    pub fn push_str(&mut self, data: &str) -> Result<usize, &'static str> {
        self.push_bytes(data.as_bytes())
    }

    pub fn push_bytes(&mut self, data: &[u8]) -> Result<usize, &'static str> {
        let len = data.len();
        // align 8
        let start = self.stack_top - len;
        let start = start & !7;
        if start < self.stack_bottom {
            return Err("Stack Overflow");
        }
        unsafe {
            self.stack_top = start;
            let ptr = self.stack_top as *mut u8;
            ptr.copy_from_nonoverlapping(data.as_ptr(), len);
        }
        trace!(
            "stack top: {:#x}",
            self.virt_stack_top - (FRAME_SIZE - (self.stack_top - self.stack_bottom))
        );
        Ok(self.virt_stack_top - (FRAME_SIZE - (self.stack_top - self.stack_bottom)))
    }

    pub fn align_to(&mut self, align: usize) -> Result<usize, &'static str> {
        let start = self.stack_top & !(align - 1);
        if start < self.stack_bottom {
            return Err("Stack Overflow");
        }
        self.stack_top = start;
        Ok(self.virt_stack_top - (FRAME_SIZE - (self.stack_top - self.stack_bottom)))
    }
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
    for (v_addr, target) in p_table.get_record().into_iter() {
        trace!("v_addr: {:?}, target: {}", v_addr, target);
        let (phy, flag, page_size) = p_table.query(v_addr).unwrap();
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
            let mut flags = flag.clone();
            if !flag.contains(MappingFlags::V) {
                // if flags is not valid, we just map it
                address_space.map(v_addr, phy, page_size, flags).unwrap();
                if target {
                    address_space.get_record_mut().insert(v_addr, true);
                }
                continue;
            }
            if flag.contains(MappingFlags::W) {
                flags -= MappingFlags::W;
                flags |= MappingFlags::RSD; // we use the RSD flag to indicate that this page is a cow page
                // update parent's flag and clear dirty
                p_table.modify_pte_flags(v_addr, flags, false).unwrap();
            }
            address_space.map(v_addr, phy, page_size, flags).unwrap();
            // add ref for alloc page
            if target {
                for i in 0..usize::from(page_size) / FRAME_SIZE {
                    let page_number = (phy + FRAME_SIZE * i).as_usize() >> FRAME_BITS;
                    FRAME_REF_MANAGER.lock().get_ref(page_number);
                    FRAME_REF_MANAGER.lock().add_ref(page_number);
                }
                address_space.get_record_mut().insert(v_addr, true);
            }
        }
    }
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

            let vaddr = VirtAddr::from(start_addr).align_down_4k();
            let end_vaddr = VirtAddr::from(end_addr).align_up_4k();
            let len = end_vaddr.as_usize() - vaddr.as_usize();
            warn!(
                "load segment: {:#x} - {:#x} -> {:#x}-{:#x}, permission: {:?}",
                start_addr,
                end_addr,
                vaddr.as_usize(),
                end_vaddr.as_usize(),
                permission
            );
            let mut data =
                &elf.input[ph.offset() as usize..(ph.offset() + ph.file_size()) as usize];
            let map_info = address_space
                .map_region_no_target(vaddr, len, permission, true, false)
                .unwrap();
            // copy data
            let mut page_offset = start_addr & (FRAME_SIZE - 1);
            map_info
                .into_iter()
                .for_each(|(vir, phy, page_size)| unsafe {
                    trace!("{:#x} {:#x} {:#x?}", vir, phy, page_size);
                    let size: usize = page_size.into();
                    // let min = min(size, data.len());
                    let min = min(size - page_offset, data.len());
                    let dst = (phy.as_usize() + page_offset) as *mut u8;
                    core::ptr::copy(data.as_ptr(), dst, min);
                    data = &data[min..];
                    page_offset = (page_offset + min) & (FRAME_SIZE - 1);
                });
        }
    }
    // 地址向上取整对齐4
    let ceil_addr = align_up_4k(break_addr);
    // 留出一个用户栈的位置+隔离页
    let top = ceil_addr + USER_STACK_SIZE + FRAME_SIZE; // 8k +4k

    warn!("user stack: {:#x} - {:#x}", top - USER_STACK_SIZE, top);
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
    let heap_bottom = top;
    // align to 4k
    warn!("trap context: {:#x} - {:#x}", TRAP_CONTEXT_BASE, TRAMPOLINE);
    address_space
        .map_region_no_target(
            VirtAddr::from(TRAP_CONTEXT_BASE),
            TRAMPOLINE - TRAP_CONTEXT_BASE,
            "RWVAD".into(),
            true,
            false,
        )
        .unwrap();
    warn!("TRAMPOLINE: {:#x} - {:#x}", TRAMPOLINE, TRAMPOLINE + FRAME_SIZE);
    address_space
        .map_region(
            VirtAddr::from(TRAMPOLINE),
            PhysAddr::from(strampoline as usize),
            FRAME_SIZE,
            "RXVAD".into(),
            true,
        )
        .unwrap();

    // TODO! dyn link
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
        Ok(elf_addr.virtual_addr())
    } else {
        warn!("elf: no phdr found, tls might not work");
        Err(ELFError::NoEntrySegment)
        // Ok(0)
    }
        .unwrap_or(0);
    warn!(
        "entry: {:#x}, phdr:{:#x}",
        elf.header.pt2.entry_point(),
        res
    );
    Ok(ELFInfo {
        address_space,
        entry: elf.header.pt2.entry_point() as usize,
        stack_top: top,
        heap_bottom,
        ph_num: elf.header.pt2.ph_count() as usize,
        ph_entry_size: elf.header.pt2.ph_entry_size() as usize,
        ph_drift: res as usize,
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
