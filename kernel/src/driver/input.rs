use alloc::collections::VecDeque;
use alloc::sync::Arc;

use hashbrown::HashMap;
use spin::Once;
use virtio_drivers::device::input::VirtIOInput;
use virtio_drivers::transport::mmio::MmioTransport;

use kernel_sync::Mutex;
use syscall_table::syscall_func;

use crate::driver::hal::HalImpl;
use crate::driver::DeviceBase;
use crate::task::schedule::schedule;
use crate::task::{current_task, Task, TaskState};

pub static mut INPUT_DEVICE: Once<HashMap<&str, Arc<InputDriver>>> = Once::new();

pub struct InputDriver {
    inner: Mutex<InputDriverInner>,
}

struct InputDriverInner {
    max_events: u32,
    driver: VirtIOInput<HalImpl, MmioTransport>,
    events: VecDeque<u64>,
    wait_queue: VecDeque<Arc<Task>>,
}

impl InputDriver {
    pub fn new(driver: VirtIOInput<HalImpl, MmioTransport>, max_events: u32) -> Self {
        let driver = InputDriver {
            inner: Mutex::new(InputDriverInner {
                max_events,
                driver,
                events: VecDeque::with_capacity(max_events as usize),
                wait_queue: VecDeque::new(),
            }),
        };
        driver
    }

    pub fn read_event(&self) -> u64 {
        loop {
            let mut inner = self.inner.lock();
            if let Some(event) = inner.events.pop_front() {
                return event;
            }
            let process = current_task().unwrap();
            process.update_state(TaskState::Waiting);
            inner.wait_queue.push_back(process.clone());
            drop(inner);
            schedule();
        }
    }

    pub fn read_event_nonblock(&self) -> Option<u64> {
        let mut inner = self.inner.lock();
        inner.events.pop_front()
    }

    pub fn is_empty(&self) -> bool {
        let inner = self.inner.lock();
        inner.events.is_empty()
    }
}

unsafe impl Send for InputDriver {}

unsafe impl Sync for InputDriver {}

impl DeviceBase for InputDriver {
    fn hand_irq(&self) {
        let mut inner = self.inner.lock();
        inner.driver.ack_interrupt();
        let mut count = 0;
        while let Some(event) = inner.driver.pop_pending_event() {
            let result =
                (event.event_type as u64) << 48 | (event.code as u64) << 32 | (event.value) as u64;
            warn!("event: {:x}", result);
            if inner.events.len() >= inner.max_events as usize {
                // remove the first event
                inner.events.pop_front();
            }
            inner.events.push_back(result);
            count += 1;
        }
        while !inner.wait_queue.is_empty() && count > 0 {
            let process = inner.wait_queue.pop_front().unwrap();
            process.update_state(TaskState::Ready);
            let mut guard = crate::task::TASK_MANAGER.lock();
            guard.push_back(process);
            count -= 1;
        }
    }
}

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
    let (keyboard, mouse) = unsafe {
        let kb = INPUT_DEVICE.get().unwrap().get("keyboard").unwrap().clone();
        let mouse = INPUT_DEVICE.get().unwrap().get("mouse").unwrap().clone();
        (kb, mouse)
    };
    if !keyboard.is_empty() {
        keyboard.read_event()
    } else if !mouse.is_empty() {
        mouse.read_event()
    } else {
        0
    }
}
