use page_table::addr::{align_up_4k, PhysAddr, VirtAddr};

use syscall_table::syscall_func;

use crate::driver::gpu::GPU_DEVICE;
use crate::task::current_process;

const FB_VADDR: usize = 0x1000_0000;

#[syscall_func(2000)]
pub fn sys_framebuffer() -> isize {
    let fb = GPU_DEVICE.get().unwrap().get_framebuffer();
    let len = fb.len();
    // println!("[kernel] FrameBuffer: addr 0x{:X}, len {}", fb.as_ptr() as usize , len);
    let phy_addr = PhysAddr::from(fb.as_ptr() as usize);
    assert!(phy_addr.is_aligned_4k());
    let virt_addr = VirtAddr::from(FB_VADDR);
    let current_process = current_process().unwrap();
    let mut inner = current_process.access_inner();
    inner
        .address_space
        .map_region(virt_addr, phy_addr, align_up_4k(len), "RWUVAD".into(), true)
        .unwrap();
    FB_VADDR as isize
}

#[syscall_func(2001)]
pub fn sys_framebuffer_flush() -> isize {
    GPU_DEVICE.get().unwrap().flush();
    0
}
