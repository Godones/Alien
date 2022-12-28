use crate::config::{FRAME_SIZE, MEMORY_END, MMIO, USER_STACK_SIZE};
use crate::memory::frame::{addr_to_frame, frame_alloc};
use alloc::sync::Arc;
use core::intrinsics::forget;
use lazy_static::lazy_static;
use page_table::{
    ap_from_str, vpn_f_c_range, AddressSpace, Area, AreaPermission, PageManager, PPN, VPN,
};
use spin::RwLock;
use xmas_elf::program;

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
    kernel_space.push(text_area);
    kernel_space.push(rodata_area);
    kernel_space.push(data_area);
    kernel_space.push(bss_area);
    kernel_space.push(heap_area);

    for pair in MMIO {
        let vpn_range = vpn_f_c_range!(pair.0, pair.0 + pair.1);
        info!("mmio range: {:x?}", vpn_range);
        let mmio_area = Area::new(vpn_range.clone(), Some(vpn_range), ap_from_str!("rw"));
        kernel_space.push(mmio_area);
    }

    info!("build kernel address space success");
}

pub struct ELFResult {
    pub address_space: AddressSpace,
    pub entry: usize,
    pub stack_top: usize,
}

pub fn build_elf_address_space(elf: &[u8]) -> Result<ELFResult, &'static str> {
    let mut address_space = AddressSpace::new(Arc::new(PageAllocator));
    const ELF_MAGIC: [u8; 4] = [0x7f, b'E', b'L', b'F'];
    if elf[0..4] != ELF_MAGIC {
        return Err("NotELF");
    }
    let elf = xmas_elf::ElfFile::new(elf);
    if elf.is_err() {
        return Err("NotELF");
    }
    let elf = elf.unwrap();
    if elf.header.pt1.magic != ELF_MAGIC {
        return Err("FileBreak");
    }
    let mut break_addr = 0usize;
    let phs = elf.header.pt2.ph_count();
    for i in 0..phs {
        let ph = elf.program_header(i)?;
        let p_type = ph.get_type()?;
        if p_type == program::Type::Load {
            let start_addr = ph.virtual_addr() as usize;
            let end_addr = start_addr + ph.mem_size() as usize;
            // 记录程序地址空间的最大地址
            break_addr = end_addr;
            let vpn_s = vpn_f_c_range!(start_addr, end_addr);
            let mut permission = AreaPermission::U;
            let ph_flags = ph.flags();
            if ph_flags.is_read() {
                permission |= AreaPermission::R;
            }
            if ph_flags.is_write() {
                permission |= AreaPermission::W;
            }
            if ph_flags.is_execute() {
                permission |= AreaPermission::X;
            }
            let area = Area::new(vpn_s, None, permission);
            let data = &elf.input[ph.offset() as usize..(ph.offset() + ph.file_size()) as usize];
            address_space.push_with_data(area, data);
        }
    }
    // 地址向上取整对齐4k
    let ceil_addr = VPN::ceil_address(break_addr).0;
    // 留出一个用户栈的位置+隔离页
    let top = ceil_addr + USER_STACK_SIZE + FRAME_SIZE;
    Ok(ELFResult {
        address_space,
        entry: elf.header.pt2.entry_point() as usize,
        stack_top: top,
    })
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
