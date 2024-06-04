use alloc::{boxed::Box, format, sync::Arc, vec, vec::Vec};
use core::sync::atomic::AtomicUsize;

use config::{FRAME_BITS, FRAME_SIZE, TRAMPOLINE};
use ksync::RwLock;
use log::info;
use page_table::MappingFlags;
use platform::{config::DEVICE_SPACE, println};
use ptable::{PhysPage, VmArea, VmAreaEqual, VmAreaType, VmSpace};
use spin::Lazy;

use super::AlienResult;
use crate::frame::{FrameTracker, VmmPageAllocator};

pub static KERNEL_SPACE: Lazy<Arc<RwLock<VmSpace<VmmPageAllocator>>>> =
    Lazy::new(|| Arc::new(RwLock::new(VmSpace::<VmmPageAllocator>::new())));

extern "C" {
    fn stext();
    fn srodata();
    fn sdata();
    fn sbss();
    fn ekernel();
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
        sbss as usize, ekernel as usize
    );
    // println!("kernel eh_frame:      {:#x}-{:#x}", kernel_eh_frame as usize, kernel_eh_frame_end as usize);
    // println!("kernel eh_frame_hdr:  {:#x}-{:#x}", kernel_eh_frame_hdr as usize, kernel_eh_frame_hdr_end as usize);
    println!(
        "kernel heap:          {:#x}-{:#x}",
        ekernel as usize, memory_end
    );
    ekernel as usize
}

static KERNEL_MAP_MAX: AtomicUsize = AtomicUsize::new(0);
pub fn build_kernel_address_space(memory_end: usize) {
    kernel_info(memory_end);
    let mut kernel_space = KERNEL_SPACE.write();
    let text_area = VmAreaEqual::new(
        stext as _..srodata as _,
        MappingFlags::READ | MappingFlags::EXECUTE,
    );
    let rodata_area = VmAreaEqual::new(srodata as _..sdata as _, MappingFlags::READ);
    let sdata_area = VmAreaEqual::new(
        sdata as _..sbss as _,
        MappingFlags::READ | MappingFlags::WRITE,
    );
    let sbss_area = VmAreaEqual::new(
        sbss as _..ekernel as _,
        MappingFlags::READ | MappingFlags::WRITE,
    );
    let free_area = VmAreaEqual::new(
        ekernel as _..memory_end,
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

/// Allocate a free region in kernel space.
pub fn alloc_free_region(size: usize) -> Option<usize> {
    assert!(size > 0 && size % FRAME_SIZE == 0);
    Some(KERNEL_MAP_MAX.fetch_add(size, core::sync::atomic::Ordering::SeqCst))
}

pub fn map_area_to_kernel(area: VmArea) -> AlienResult<()> {
    let mut kernel_space = KERNEL_SPACE.write();
    kernel_space.map(VmAreaType::VmArea(area)).unwrap();
    Ok(())
}

pub fn unmap_region_from_kernel(addr: usize) -> Result<(), &'static str> {
    assert_eq!(addr % FRAME_SIZE, 0);
    let mut kernel_space = KERNEL_SPACE.write();
    kernel_space.unmap(addr).unwrap();
    Ok(())
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
        .expect(&format!("unmap kstack failed, task_id:{}", task_id));
    info!(
        "unmap task {} kstack: {:#x?}-{:#x?}",
        task_id, kstack_lower, kstack_upper
    );
    Ok(())
}
