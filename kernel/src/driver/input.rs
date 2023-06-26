use alloc::collections::VecDeque;
use alloc::sync::Arc;

use hashbrown::HashMap;
use spin::Once;
use virtio_drivers::device::input::VirtIOInput;
use virtio_drivers::transport::mmio::MmioTransport;

use kernel_sync::Mutex;
use syscall_table::syscall_func;

use crate::driver::DeviceBase;
use crate::driver::hal::HalImpl;
use crate::task::{current_process, Process, ProcessState};
use crate::task::schedule::schedule;

pub static mut INPUT_DEVICE: Once<HashMap<&str, Arc<InputDriver>>> = Once::new();

pub struct InputDriver {
    inner: Mutex<InputDriverInner>,
}

struct InputDriverInner {
    driver: VirtIOInput<HalImpl, MmioTransport>,
    events: VecDeque<u64>,
    wait_queue: VecDeque<Arc<Process>>,
}

impl InputDriver {
    pub fn new(driver: VirtIOInput<HalImpl, MmioTransport>) -> Self {
        let driver = InputDriver {
            inner: Mutex::new(InputDriverInner {
                driver,
                events: VecDeque::new(),
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
            let process = current_process().unwrap();
            process.update_state(ProcessState::Waiting);
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
            let result = (event.event_type as u64) << 48
                | (event.code as u64) << 32
                | (event.value) as u64;
            warn!("event: {:x}", result);
            inner.events.push_back(result);
            count += 1;
        }
        while !inner.wait_queue.is_empty() && count > 0 {
            let process = inner.wait_queue.pop_front().unwrap();
            process.update_state(ProcessState::Ready);
            let mut guard = crate::task::PROCESS_MANAGER.lock();
            guard.push_back(process);
            count -= 1;
        }
    }
}

#[syscall_func(2002)]
pub fn sys_event_get() -> isize {
    let (keyboard, mouse) = unsafe {
        let kb = INPUT_DEVICE.get().unwrap().get("keyboard").unwrap().clone();
        let mouse = INPUT_DEVICE.get().unwrap().get("mouse").unwrap().clone();
        (kb, mouse)
    };
    //let input=INPUT_CONDVAR.clone();
    //read_input_event() as isize
    if !keyboard.is_empty() {
        keyboard.read_event() as isize
    } else if !mouse.is_empty() {
        mouse.read_event() as isize
    } else {
        0
    }
}