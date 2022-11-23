use crate::config::{MEMORY_END, RISCV_UART_ADDR, RISCV_UART_RANG};
use crate::mm::frame::{addr_to_frame, frame_alloc};
use alloc::sync::Arc;
use core::intrinsics::forget;
use lazy_static::lazy_static;
use page_table::{
    ap_from_str, vpn_f_c_range, AddressSpace, Area, AreaPermission, PageManager, PPN, VPN,
};
use spin::RwLock;

lazy_static! {
    pub static ref KERNEL_SPACE: Arc<RwLock<AddressSpace>> =
        Arc::new(RwLock::new(AddressSpace::new(Arc::new(PageAllocator))));
}

/// 建立内核页表
pub fn build_kernel_address_space() {
    info!("build kernel address space");
    #[allow(unused)]
    extern "C" {
        fn stext();
        fn srodata();
        fn sdata();
        fn sbss();
        fn ekernel();
    }
    let mut kernel_space = KERNEL_SPACE.write();
    let vpn_range = vpn_f_c_range!(stext as usize, srodata as usize);
    info!("kernel text range: {:x?}", vpn_range);
    let text_area = Area::new(vpn_range.clone(), Some(vpn_range), ap_from_str!("rx"));
    let vpn_range = vpn_f_c_range!(srodata as usize, sdata as usize);
    info!("kernel rodata range: {:x?}", vpn_range);
    let rodata_area = Area::new(vpn_range.clone(), Some(vpn_range), AreaPermission::R);
    let vpn_range = vpn_f_c_range!(sdata as usize, sbss as usize);
    info!("kernel data range: {:x?}", vpn_range);
    let data_area = Area::new(vpn_range.clone(), Some(vpn_range), ap_from_str!("rw"));
    let vpn_range = vpn_f_c_range!(sbss as usize, ekernel as usize);
    info!("kernel bss range: {:x?}", vpn_range);
    let bss_area = Area::new(vpn_range.clone(), Some(vpn_range), ap_from_str!("rw"));
    let vpn_range = vpn_f_c_range!(ekernel as usize, MEMORY_END);
    info!("kernel heap range: {:x?}", vpn_range);
    let heap_area = Area::new(vpn_range.clone(), Some(vpn_range), ap_from_str!("rw"));
    // 映射UART区域
    let vpn_range = vpn_f_c_range!(RISCV_UART_ADDR, RISCV_UART_ADDR + RISCV_UART_RANG);
    info!("uart range: {:x?}", vpn_range);
    let uart_area = Area::new(vpn_range.clone(), Some(vpn_range), ap_from_str!("rw"));
    kernel_space.push(text_area);
    kernel_space.push(rodata_area);
    kernel_space.push(data_area);
    kernel_space.push(bss_area);
    kernel_space.push(heap_area);
    kernel_space.push(uart_area);
    info!("build kernel address space success");
}

pub struct PageAllocator;

impl PageManager for PageAllocator {
    fn alloc(&self) -> Option<PPN> {
        frame_alloc().map(|frame| {
            let start = frame.start();
            trace!("PageAllocator alloc frame{:?} start:{:#x}", frame, start);
            forget(frame);
            PPN::ceil_address(start)
        })
    }
    fn dealloc(&self, ppn: PPN) {
        let frame = addr_to_frame(ppn.to_address());
        trace!("PageAllocator dealloc frame {:?}", frame);
        drop(frame);
    }
}

#[allow(unused)]
pub fn test_page_allocator() {
    let allocator = PageAllocator;
    let ppn = allocator.alloc().unwrap();
    allocator.dealloc(ppn);
    println!("page allocator test passed");
}
