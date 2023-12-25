use crate::frame::{alloc_frames, VmmPageAllocator};
use alloc::sync::Arc;
use config::{FRAME_SIZE, MMIO};
use core::sync::atomic::AtomicUsize;
use ksync::RwLock;
use page_table::riscv::Sv39PageTable;
use spin::Lazy;

pub use memory_addr::{PhysAddr, VirtAddr};
pub use page_table::MappingFlags;
use page_table::PageSize;

pub static KERNEL_SPACE: Lazy<Arc<RwLock<Sv39PageTable<VmmPageAllocator>>>> = Lazy::new(|| {
    Arc::new(RwLock::new(
        Sv39PageTable::<VmmPageAllocator>::try_new().unwrap(),
    ))
});

extern "C" {
    fn stext();
    fn srodata();
    fn sdata();
    fn sbss();
    fn ekernel();
    // fn kernel_eh_frame();
    // fn kernel_eh_frame_end();
    // fn kernel_eh_frame_hdr();
    // fn kernel_eh_frame_hdr_end();
}

/// Return the physical address of the root page table.
pub fn kernel_pgd() -> usize {
    KERNEL_SPACE.read().root_paddr().as_usize()
}

static KERNEL_MAP_MAX: AtomicUsize = AtomicUsize::new(0);
pub fn build_kernel_address_space(memory_end: usize) {
    let mut kernel_space = KERNEL_SPACE.write();
    kernel_space
        .map_region(
            VirtAddr::from(stext as usize),
            PhysAddr::from(stext as usize),
            srodata as usize - stext as usize,
            MappingFlags::READ | MappingFlags::EXECUTE,
            true,
        )
        .unwrap();
    kernel_space
        .map_region(
            VirtAddr::from(srodata as usize),
            PhysAddr::from(srodata as usize),
            sdata as usize - srodata as usize,
            MappingFlags::READ,
            true,
        )
        .unwrap();
    kernel_space
        .map_region(
            VirtAddr::from(sdata as usize),
            PhysAddr::from(sdata as usize),
            sbss as usize - sdata as usize,
            MappingFlags::READ | MappingFlags::WRITE,
            true,
        )
        .unwrap();
    kernel_space
        .map_region(
            VirtAddr::from(sbss as usize),
            PhysAddr::from(sbss as usize),
            ekernel as usize - sbss as usize,
            MappingFlags::READ | MappingFlags::WRITE | MappingFlags::EXECUTE,
            true,
        )
        .unwrap();
    kernel_space
        .map_region(
            VirtAddr::from(ekernel as usize),
            PhysAddr::from(ekernel as usize),
            memory_end - ekernel as usize,
            MappingFlags::READ | MappingFlags::WRITE,
            true,
        )
        .unwrap();
    // kernel_space
    //     .map_region(
    //         VirtAddr::from(TRAMPOLINE),
    //         PhysAddr::from(strampoline as usize),
    //         FRAME_SIZE,
    //         MappingFlags::READ | MappingFlags::EXECUTE,
    //         true,
    //     )
    //     .unwrap();
    for pair in MMIO {
        kernel_space
            .map_region(
                VirtAddr::from(pair.0),
                PhysAddr::from(pair.0),
                pair.1,
                MappingFlags::READ | MappingFlags::WRITE,
                true,
            )
            .unwrap();
    }
    KERNEL_MAP_MAX.store(memory_end, core::sync::atomic::Ordering::SeqCst);
}

pub fn alloc_free_region(size: usize) -> Option<usize> {
    assert!(size > 0 && size % FRAME_SIZE == 0);
    Some(KERNEL_MAP_MAX.fetch_add(size, core::sync::atomic::Ordering::SeqCst))
}

pub fn map_region_to_kernel(
    addr: usize,
    size: usize,
    flags: MappingFlags,
) -> Result<(), &'static str> {
    assert!(size > 0 && size % FRAME_SIZE == 0);
    assert_eq!(addr % FRAME_SIZE, 0);
    let mut kernel_space = KERNEL_SPACE.write();
    let mut addr = addr;
    for _ in 0..size / FRAME_SIZE {
        let phys_addr = alloc_frames(1);
        kernel_space
            .map(
                VirtAddr::from(addr),
                PhysAddr::from(phys_addr as usize),
                PageSize::Size4K,
                flags,
            )
            .unwrap();
        addr += FRAME_SIZE;
    }
    Ok(())
}

pub fn unmap_region_from_kernel(addr: usize, size: usize) -> Result<(), &'static str> {
    assert!(size > 0 && size % FRAME_SIZE == 0);
    assert_eq!(addr % FRAME_SIZE, 0);
    let mut kernel_space = KERNEL_SPACE.write();
    let mut addr = addr;
    for _ in 0..size / FRAME_SIZE {
        kernel_space.unmap(VirtAddr::from(addr)).unwrap();
        addr += FRAME_SIZE;
    }
    Ok(())
}

pub fn query_kernel_space(addr: usize) -> Option<usize> {
    let kernel_space = KERNEL_SPACE.read();
    kernel_space
        .query(VirtAddr::from(addr))
        .ok()
        .map(|(x, _, _)| x.as_usize())
}
