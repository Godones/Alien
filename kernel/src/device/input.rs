use alloc::sync::Arc;

use spin::Once;

use syscall_table::syscall_func;

use crate::interrupt::DeviceBase;
use crate::task::current_task;

pub trait InputDevice: Send + Sync + DeviceBase {
    fn is_empty(&self) -> bool;
    fn read_event_with_block(&self) -> u64;
    fn read_event_without_block(&self) -> Option<u64>;
}

pub static KEYBOARD_INPUT_DEVICE: Once<Arc<dyn InputDevice>> = Once::new();
pub static MOUSE_INPUT_DEVICE: Once<Arc<dyn InputDevice>> = Once::new();

#[allow(unused)]
pub fn init_keyboard_input_device(input_device: Arc<dyn InputDevice>) {
    KEYBOARD_INPUT_DEVICE.call_once(|| input_device);
}

#[allow(unused)]
pub fn init_mouse_input_device(input_device: Arc<dyn InputDevice>) {
    MOUSE_INPUT_DEVICE.call_once(|| input_device);
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
    let process = current_task().unwrap();
    let user_buffer = process.transfer_buffer(event_buf, len);
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
        let kb = KEYBOARD_INPUT_DEVICE.get().unwrap().clone();
        let mouse = MOUSE_INPUT_DEVICE.get().unwrap().clone();
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
