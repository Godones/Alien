//! GUI 相关的系统调用
use page_table::addr::{align_up_4k, PhysAddr, VirtAddr};
use crate::task::current_task;


use devices::GPU_DEVICE;

const FB_VADDR: usize = 0x1000_0000;

/// 一个系统调用，用于获取一段帧缓存。执行成功后返回帧缓存的首地址，Alien 中默认该地址为 FB_VADDR (0x1000_0000)
#[syscall_func(2000)]
pub fn sys_framebuffer() -> isize {
    let fb = GPU_DEVICE.get().unwrap().get_framebuffer();
    let len = fb.len();
    // println!("[kernel] FrameBuffer: addr 0x{:X}, len {}", fb.as_ptr() as usize , len);
    let phy_addr = PhysAddr::from(fb.as_ptr() as usize);
    assert!(phy_addr.is_aligned_4k());
    let virt_addr = VirtAddr::from(FB_VADDR);
    let current_process = current_task().unwrap();
    let inner = current_process.access_inner();
    inner
        .address_space
        .lock()
        .map_region(virt_addr, phy_addr, align_up_4k(len), "RWUVAD".into(), true)
        .unwrap();
    FB_VADDR as isize
}

/// 一个系统调用，用于刷新帧缓存。执行成功后返回 0。
#[syscall_func(2001)]
pub fn sys_framebuffer_flush() -> isize {
    GPU_DEVICE.get().unwrap().flush();
    0
}
