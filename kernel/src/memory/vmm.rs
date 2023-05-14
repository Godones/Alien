use alloc::sync::Arc;
use core::fmt::{Debug, Formatter};
use core::intrinsics::forget;

use lazy_static::lazy_static;
use page_table::{
    AddressSpace, ap_from_str, Area, AreaPermission, PageManager, PPN, VPN, vpn_f_c_range,
};
use spin::RwLock;
use virtio_drivers::PAGE_SIZE;
use xmas_elf::program;

use crate::config::{FRAME_SIZE, MEMORY_END, MMIO, TRAMPOLINE, TRAP_CONTEXT_BASE, USER_STACK_SIZE};
use crate::fs::DISK_DEVICE;
use crate::memory::frame::{addr_to_frame, frame_alloc};

lazy_static! {
    pub static ref KERNEL_SPACE: Arc<RwLock<AddressSpace>> =
        Arc::new(RwLock::new(AddressSpace::new(Arc::new(PageAllocator))));
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

pub fn kernel_info() {
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
    println!("kernel heap:   {:#x}-{:#x}", ekernel as usize, MEMORY_END);
}

/// 建立内核页表
pub fn build_kernel_address_space() {
    info!("build kernel address space");
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
    let tramppoline_area = Area::new(
        vpn_f_c_range!(TRAMPOLINE, TRAMPOLINE + FRAME_SIZE),
        Some(vpn_f_c_range!(
            strampoline as usize,
            strampoline as usize + FRAME_SIZE
        )),
        ap_from_str!("rx"),
    );
    kernel_space.push(text_area);
    kernel_space.push(rodata_area);
    kernel_space.push(data_area);
    kernel_space.push(bss_area);
    kernel_space.push(heap_area);
    kernel_space.push(tramppoline_area);
    for pair in MMIO {
        let vpn_range = vpn_f_c_range!(pair.0, pair.0 + pair.1);
        info!("mmio range: {:x?}", vpn_range);
        let mmio_area = Area::new(vpn_range.clone(), Some(vpn_range), ap_from_str!("rw"));
        kernel_space.push(mmio_area);
    }
    info!("build kernel address space success");
}


const DISK_START_VA: usize = 0x90_000_000;

pub fn tmp_insert_disk_map(size: usize) -> usize {
    let mut kernel_space = KERNEL_SPACE.write();
    let vpn_range = vpn_f_c_range!(DISK_START_VA, DISK_START_VA + size);
    println!("disk range: {:x?}", vpn_range);
    let disk_area = Area::new(vpn_range.clone(), None, ap_from_str!("rw"));
    kernel_space.tmp_push(disk_area, false);
    DISK_START_VA
}

pub fn tmp_solve_disk_map_page_fault(va: usize) {
    assert!(va >= DISK_START_VA, "va:{:#x}", va);
    let kernel_space = KERNEL_SPACE.write();
    kernel_space.tmp_make_valid(VPN::floor_address(va));
    let phy = kernel_space.virtual_to_physical(va).unwrap();
    trace!("solve disk map page fault, va:{:#x}, phy:{:#x}", va, phy);
    let disk = DISK_DEVICE.get().unwrap();
    let mut device = disk.device.lock();
    let va_to_read = va - DISK_START_VA;
    trace!("va to read: {:#x}", va_to_read);
    let buf = unsafe {
        core::slice::from_raw_parts_mut(phy as *mut u8, PAGE_SIZE)
    };
    let start_block = va_to_read / 512;
    let end_block = (va_to_read + PAGE_SIZE) / 512;
    let mut offset = 0;
    trace!("start block: {}, end block: {}", start_block, end_block);
    for i in start_block..end_block {
        let read_buf = &mut buf[offset..offset + 512];
        device.read_block(i, read_buf).unwrap();
        offset += 512;
    }
    trace!("read disk success");
}


pub struct ELFInfo {
    pub address_space: AddressSpace,
    pub entry: usize,
    pub stack_top: usize,
    pub heap_bottom: usize,
}

impl Debug for ELFInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!(
            "ELFInfo {{ address_space: {:#x?}, entry: {:#x}, stack_top: {:#x} }}",
            self.address_space.root_ppn().unwrap().0,
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

pub fn build_elf_address_space(elf: &[u8]) -> Result<ELFInfo, ELFError> {
    let mut address_space = AddressSpace::new(Arc::new(PageAllocator));
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
    // 地址向上取整对齐4
    let ceil_addr = VPN::ceil_address(break_addr).to_address();
    // 留出一个用户栈的位置+隔离页
    let top = ceil_addr + USER_STACK_SIZE + FRAME_SIZE; // 8k +4k

    // map user stack
    let vpn_range = vpn_f_c_range!(top - USER_STACK_SIZE, top);
    let stack_area = Area::new(vpn_range, None, ap_from_str!("rwu"));
    address_space.push(stack_area);

    // todo!(heap)
    let heap_bottom = top; // align to 4k
    // map trap context
    let vpn_range = vpn_f_c_range!(TRAP_CONTEXT_BASE, TRAMPOLINE);
    let trap_area = Area::new(vpn_range, None, ap_from_str!("rw"));
    address_space.push(trap_area);

    let tramppoline_area = Area::new(
        vpn_f_c_range!(TRAMPOLINE, TRAMPOLINE + FRAME_SIZE),
        Some(vpn_f_c_range!(
            strampoline as usize,
            strampoline as usize + FRAME_SIZE
        )),
        ap_from_str!("rx"),
    );
    address_space.push(tramppoline_area);

    Ok(ELFInfo {
        address_space,
        entry: elf.header.pt2.entry_point() as usize,
        stack_top: top,
        heap_bottom,
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
