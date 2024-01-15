use alloc::sync::Arc;
use core::sync::atomic::AtomicUsize;
use page_table::addr::{PhysAddr, VirtAddr};
use page_table::pte::MappingFlags;
use page_table::table::Sv39PageTable;
use spin::Lazy;
use config::{FRAME_BITS, FRAME_SIZE, TRAMPOLINE};
use ksync::RwLock;
use platform::config::MMIO;
use crate::frame::VmmPageAllocator;

#[allow(unused)]
extern "C" {
    fn stext();
    fn srodata();
    fn sdata();
    fn sbss();
    fn ekernel();
    fn strampoline();
    fn sinit();
    fn einit();
    // fn kernel_eh_frame();
    // fn kernel_eh_frame_end();
    // fn kernel_eh_frame_hdr();
    // fn kernel_eh_frame_hdr_end();
}

fn kernel_info(memory_end: usize) {
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
}


pub static KERNEL_SPACE: Lazy<Arc<RwLock<Sv39PageTable<VmmPageAllocator>>>> = Lazy::new(|| {
    Arc::new(RwLock::new(
        Sv39PageTable::<VmmPageAllocator>::try_new().unwrap(),
    ))
});

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
                VirtAddr::from(pair.0),
                PhysAddr::from(pair.0),
                pair.1,
                "RWVAD".into(),
                true,
            )
            .unwrap();
        println!("map mmio: {:#x?}-{:#x?}", pair.0, pair.0 + pair.1);
    }
}

static KERNEL_MAP_MAX: AtomicUsize = AtomicUsize::new(0);
pub fn kernel_pgd() -> usize {
    KERNEL_SPACE.read().root_paddr().as_usize()
}

pub fn kernel_satp() -> usize {
    8usize << 60 | (KERNEL_SPACE.read().root_paddr().as_usize() >> FRAME_BITS)
}

pub fn alloc_kernel_free_region(size: usize) -> usize {
    assert!(size > 0 && size % FRAME_SIZE == 0);
    KERNEL_MAP_MAX.fetch_add(size, core::sync::atomic::Ordering::SeqCst)
}

pub fn map_region_to_kernel(addr: usize, size: usize, flags: MappingFlags){
    let mut kernel_space = KERNEL_SPACE.write();
    kernel_space
        .map_region(
            VirtAddr::from(addr),
            PhysAddr::from(addr),
            size,
            flags,
            true,
        )
        .unwrap();
}
pub fn query_kernel_space(addr: usize) -> Option<usize> {
    let kernel_space = KERNEL_SPACE.read();
    kernel_space
        .query(VirtAddr::from(addr))
        .ok()
        .map(|(x, _, _)| x.as_usize())
}
