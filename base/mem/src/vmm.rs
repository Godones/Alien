use alloc::{boxed::Box, sync::Arc, vec, vec::Vec};
use core::sync::atomic::AtomicUsize;

use arch::sfence_vma_all;
use config::{FRAME_BITS, FRAME_SIZE, TRAMPOLINE};
use ksync::RwLock;
use log::info;
use page_table::MappingFlags;
use platform::{config::DEVICE_SPACE, println};
use ptable::{PhysPage, VmArea, VmAreaEqual, VmAreaType, VmSpace};
use spin::Lazy;

use super::{alloc_frame_trackers, AlienResult};
use crate::frame::{FrameTracker, VmmPageAllocator};

pub static KERNEL_SPACE: Lazy<Arc<RwLock<VmSpace<VmmPageAllocator>>>> =
    Lazy::new(|| Arc::new(RwLock::new(VmSpace::<VmmPageAllocator>::new())));

extern "C" {
    fn stext();
    fn srodata();
    fn sdata();
    fn sbss();
    fn sheap();
    fn sinit();
    fn einit();

    // fn kernel_eh_frame();
    // fn kernel_eh_frame_end();
    // fn kernel_eh_frame_hdr();
    // fn kernel_eh_frame_hdr_end();
}

pub fn kernel_info(memory_end: usize) -> usize {
    println!(
        "kernel text:          {:#x}-{:#x}",
        stext as usize, srodata as usize
    );
    println!(
        "kernel rodata:        {:#x}-{:#x}",
        srodata as usize, sdata as usize
    );
    println!(
        "kernel init_array:    {:#x}-{:#x}",
        sinit as usize, einit as usize
    );
    println!(
        "kernel data:          {:#x}-{:#x}",
        sdata as usize, sbss as usize
    );
    println!(
        "kernel bss:           {:#x}-{:#x}",
        sbss as usize, sheap as usize
    );
    // println!("kernel eh_frame:      {:#x}-{:#x}", kernel_eh_frame as usize, kernel_eh_frame_end as usize);
    // println!("kernel eh_frame_hdr:  {:#x}-{:#x}", kernel_eh_frame_hdr as usize, kernel_eh_frame_hdr_end as usize);
    println!(
        "kernel heap:          {:#x}-{:#x}",
        sheap as usize, memory_end
    );
    sheap as usize
}

static KERNEL_MAP_MAX: AtomicUsize = AtomicUsize::new(0);
pub fn build_kernel_address_space(memory_end: usize) {
    kernel_info(memory_end);
    let mut kernel_space = KERNEL_SPACE.write();
    let text_area = VmAreaEqual::new(
        stext as _..srodata as _,
        MappingFlags::READ | MappingFlags::EXECUTE | MappingFlags::WRITE,
    );
    let rodata_area = VmAreaEqual::new(srodata as _..sdata as _, MappingFlags::READ);
    let sdata_area = VmAreaEqual::new(
        sdata as _..sbss as _,
        MappingFlags::READ | MappingFlags::WRITE,
    );
    let sbss_area = VmAreaEqual::new(
        sbss as _..sheap as _,
        MappingFlags::READ | MappingFlags::WRITE,
    );
    let free_area = VmAreaEqual::new(
        sheap as _..memory_end,
        MappingFlags::READ | MappingFlags::WRITE,
    );

    let trampoline_area = VmArea::new(
        TRAMPOLINE..(TRAMPOLINE + FRAME_SIZE),
        MappingFlags::READ | MappingFlags::EXECUTE,
        vec![Box::new(FrameTracker::create_trampoline())],
    );
    kernel_space
        .map(VmAreaType::VmAreaEqual(text_area))
        .unwrap();
    kernel_space
        .map(VmAreaType::VmAreaEqual(rodata_area))
        .unwrap();
    kernel_space
        .map(VmAreaType::VmAreaEqual(sdata_area))
        .unwrap();
    kernel_space
        .map(VmAreaType::VmAreaEqual(sbss_area))
        .unwrap();
    kernel_space
        .map(VmAreaType::VmAreaEqual(free_area))
        .unwrap();
    kernel_space
        .map(VmAreaType::VmArea(trampoline_area))
        .unwrap();

    for pair in DEVICE_SPACE {
        let io_area = VmAreaEqual::new(
            pair.1..pair.1 + pair.2,
            MappingFlags::READ | MappingFlags::WRITE,
        );
        kernel_space.map(VmAreaType::VmAreaEqual(io_area)).unwrap();
        println!("map {}: {:#x?}-{:#x?}", pair.0, pair.1, pair.1 + pair.2);
    }
    KERNEL_MAP_MAX.store(memory_end, core::sync::atomic::Ordering::SeqCst);
}

/// Return the physical address of the root page table.
pub fn kernel_pgd() -> usize {
    KERNEL_SPACE.read().root_paddr()
}

pub fn kernel_satp() -> usize {
    8usize << 60 | (kernel_pgd() >> FRAME_BITS)
}

pub fn query_kernel_space(addr: usize) -> Option<usize> {
    let kernel_space = KERNEL_SPACE.read();
    kernel_space
        .query(addr)
        .ok()
        .map(|(phy_addr, _, _)| phy_addr.as_usize())
}

/// Layout:
///
/// TRAMPOLINE
/// |   |
/// |   |
/// Guard Page
/// |   |
/// |   |
/// Guard Page
pub fn map_kstack_for_task(task_id: usize, pages: usize) -> AlienResult<usize> {
    let kstack_base = TRAMPOLINE - (task_id + 1) * (pages + 1) * FRAME_SIZE;
    let kstack_lower = kstack_base + FRAME_SIZE;
    let kstack_upper = kstack_lower + pages * FRAME_SIZE;
    let mut phy_frames: Vec<Box<dyn PhysPage>> = vec![];
    for _ in 0..pages {
        phy_frames.push(Box::new(crate::alloc_frame_trackers(1)));
    }
    let kstack_area = VmArea::new(
        kstack_lower..kstack_upper,
        MappingFlags::READ | MappingFlags::WRITE,
        phy_frames,
    );
    let mut kernel_space = KERNEL_SPACE.write();
    kernel_space.map(VmAreaType::VmArea(kstack_area)).unwrap();
    info!(
        "task {} kstack: {:#x?}-{:#x?}",
        task_id, kstack_lower, kstack_upper
    );
    Ok(kstack_upper)
}

pub fn unmap_kstack_for_task(task_id: usize, pages: usize) -> AlienResult<()> {
    let kstack_base = TRAMPOLINE - (task_id + 1) * (pages + 1) * FRAME_SIZE;
    let kstack_lower = kstack_base + FRAME_SIZE;
    let kstack_upper = kstack_lower + pages * FRAME_SIZE;
    let mut kernel_space = KERNEL_SPACE.write();
    kernel_space
        .unmap(kstack_lower)
        .unwrap_or_else(|_| panic!("unmap kstack failed, task_id:{}", task_id));
    info!(
        "unmap task {} kstack: {:#x?}-{:#x?}",
        task_id, kstack_lower, kstack_upper
    );
    Ok(())
}

#[derive(Debug)]
pub struct VirtDomainArea {
    start: usize,
    size: usize,
}

impl VirtDomainArea {
    pub(super) fn new(start: usize, size: usize) -> Self {
        Self { start, size }
    }
    pub fn as_ptr(&self) -> *mut u8 {
        self.start as *mut u8
    }
    pub fn as_slice(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self.start as *const u8, self.size) }
    }
    pub fn as_mut_slice(&self) -> &mut [u8] {
        unsafe { core::slice::from_raw_parts_mut(self.start as *mut u8, self.size) }
    }
    pub fn len(&self) -> usize {
        self.size
    }
}

pub fn map_domain_region(size: usize) -> VirtDomainArea {
    assert_eq!(size % FRAME_SIZE, 0);
    let virt_start = KERNEL_MAP_MAX.fetch_add(size, core::sync::atomic::Ordering::Relaxed);
    // alloc physical memory and map to virtual memory
    println!(
        "[alloc_free_module_region] virt_start: {:#x}, size: {:#x}",
        virt_start, size
    );
    let mut phy_frames: Vec<Box<dyn PhysPage>> = vec![];
    for _ in 0..size / FRAME_SIZE {
        let frame = Box::new(alloc_frame_trackers(1));
        phy_frames.push(frame);
    }
    let mut kernel_space = KERNEL_SPACE.write();
    let vm_area = VmArea::new(
        virt_start..virt_start + size,
        MappingFlags::READ | MappingFlags::WRITE,
        phy_frames,
    );
    kernel_space.map(VmAreaType::VmArea(vm_area)).unwrap();
    // flush TLB
    sfence_vma_all();
    VirtDomainArea::new(virt_start, size)
}

pub fn unmap_domain_area(area: VirtDomainArea) {
    let mut kernel_space = KERNEL_SPACE.write();
    kernel_space.unmap(area.start).unwrap();
    sfence_vma_all();
}

pub fn set_memory_x(virt_addr: usize, numpages: usize) -> AlienResult<()> {
    let mut kernel_space = KERNEL_SPACE.write();
    // kernel_space.set_flags(virt_addr, numpages, MappingFlags::READ | MappingFlags::WRITE | MappingFlags::EXECUTE).unwrap();
    let mut addr = virt_addr;
    for _ in 0..numpages {
        kernel_space
            .protect(
                addr..addr + FRAME_SIZE,
                MappingFlags::READ | MappingFlags::EXECUTE,
            )
            .unwrap();
        addr += FRAME_SIZE;
    }
    Ok(())
}
