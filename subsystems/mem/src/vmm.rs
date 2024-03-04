use crate::frame::VmmPageAllocator;
use alloc::sync::Arc;
use config::{FRAME_BITS, FRAME_SIZE, TRAMPOLINE};
use constants::AlienResult;
use core::sync::atomic::AtomicUsize;
use ksync::RwLock;
use log::info;
pub use page_table::addr::{PhysAddr, VirtAddr};
pub use page_table::pte::MappingFlags;
use page_table::table::Sv39PageTable;
use platform::config::MMIO;
use platform::println;
use spin::Lazy;

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
    fn sinit();
    fn einit();
    fn strampoline();
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
                VirtAddr::from(pair.1),
                PhysAddr::from(pair.1),
                pair.2,
                "RWVAD".into(),
                true,
            )
            .unwrap();
        println!("map {}: {:#x?}-{:#x?}", pair.0, pair.1, pair.1 + pair.2);
    }
    KERNEL_MAP_MAX.store(memory_end, core::sync::atomic::Ordering::SeqCst);
}

/// Return the physical address of the root page table.
pub fn kernel_pgd() -> usize {
    KERNEL_SPACE.read().root_paddr().as_usize()
}

pub fn kernel_satp() -> usize {
    8usize << 60 | (kernel_pgd() >> FRAME_BITS)
}
pub fn alloc_free_region(size: usize) -> Option<usize> {
    assert!(size > 0 && size % FRAME_SIZE == 0);
    Some(KERNEL_MAP_MAX.fetch_add(size, core::sync::atomic::Ordering::SeqCst))
}

pub fn map_region_to_kernel(vaddr: usize, size: usize, flags: MappingFlags) -> AlienResult<()> {
    assert!(size > 0 && size % FRAME_SIZE == 0);
    assert_eq!(vaddr % FRAME_SIZE, 0);
    let mut kernel_space = KERNEL_SPACE.write();
    kernel_space
        .map_region_no_target(VirtAddr::from(vaddr), size, flags, false, false)
        .unwrap();
    Ok(())
}

pub fn unmap_region_from_kernel(addr: usize, size: usize) -> Result<(), &'static str> {
    assert!(size > 0 && size % FRAME_SIZE == 0);
    assert_eq!(addr % FRAME_SIZE, 0);
    let mut kernel_space = KERNEL_SPACE.write();
    kernel_space
        .unmap_region(VirtAddr::from(addr), size)
        .unwrap();
    Ok(())
}

pub fn query_kernel_space(addr: usize) -> Option<usize> {
    let kernel_space = KERNEL_SPACE.read();
    kernel_space
        .query(VirtAddr::from(addr))
        .ok()
        .map(|(x, _, _)| x.as_usize())
}

pub fn is_in_kernel_space(vaddr: usize, size: usize) -> bool {
    info!("[is_in_kernel_space]: {:#x}-{:#x}", vaddr, vaddr + size);
    let kernel_space = KERNEL_SPACE.read();
    let mut addr = vaddr;
    while addr < vaddr + size {
        if kernel_space.query(VirtAddr::from(addr)).is_err() {
            return false;
        }
        addr += FRAME_SIZE;
        // align to next page
        addr = (addr + FRAME_SIZE - 1) & !(FRAME_SIZE - 1);
    }
    true
}
