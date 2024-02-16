//! GUI 相关的系统调用
use crate::task::current_task;
use page_table::addr::{align_up_4k, PhysAddr, VirtAddr};

use devices::{GPU_DEVICE, KEYBOARD_INPUT_DEVICE, MOUSE_INPUT_DEVICE};

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

/// 一个系统调用函数，用于获取鼠标和键盘事件。
///
/// `sys_event_get`会将获取到的事件将保存在event_buf所指向的内存位置处，
/// 此次允许获取到的事件的最大值(即event_buf)的大小由len指出。
///
/// 函数将返回成功获取到的事件个数。
///
#[syscall_func(2002)]
pub fn sys_event_get(event_buf: *mut u64, len: usize) -> isize {
    let task = current_task().unwrap();
    let user_buffer = task.transfer_buffer(event_buf, len);
    let mut count = 0;
    for buf in user_buffer {
        let mut index = 0;
        let len = buf.len();
        while index < len {
            let event = read_event();
            if event == 0 {
                break;
            }
            buf[index] = event;
            index += 1;
            count += 1;
        }
    }
    count
}

fn read_event() -> u64 {
    let (keyboard, mouse) = {
        let kb = KEYBOARD_INPUT_DEVICE.get().unwrap();
        let mouse = MOUSE_INPUT_DEVICE.get().unwrap();
        (kb, mouse)
    };
    if !keyboard.is_empty() {
        keyboard.read_event_with_block()
    } else if !mouse.is_empty() {
        mouse.read_event_with_block()
    } else {
        0
    }
}
