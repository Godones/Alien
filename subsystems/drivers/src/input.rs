use alloc::collections::VecDeque;
use alloc::sync::Arc;
use core::ptr::NonNull;
use log::info;

use device_interface::{DeviceBase, InputDevice};
use virtio_drivers::device::input::VirtIOInput;
use virtio_drivers::transport::mmio::{MmioTransport, VirtIOHeader};

use crate::hal::HalImpl;
use ksync::Mutex;
use crate::{DRIVER_TASK, DriverTask};

pub struct VirtIOInputDriver {
    inner: Mutex<InputDriverInner>,
}

unsafe impl Send for VirtIOInputDriver {}

unsafe impl Sync for VirtIOInputDriver {}

struct InputDriverInner {
    max_events: u32,
    driver: VirtIOInput<HalImpl, MmioTransport>,
    events: VecDeque<u64>,
    wait_queue: VecDeque<Arc<dyn DriverTask>>,
}

impl VirtIOInputDriver {
    fn new(driver: VirtIOInput<HalImpl, MmioTransport>, max_events: u32) -> Self {
        let driver = VirtIOInputDriver {
            inner: Mutex::new(InputDriverInner {
                max_events,
                driver,
                events: VecDeque::with_capacity(max_events as usize),
                wait_queue: VecDeque::new(),
            }),
        };
        driver
    }

    pub fn from_addr(addr: usize, max_events: u32) -> Self {
        let header = NonNull::new(addr as *mut VirtIOHeader).unwrap();
        let transport = unsafe { MmioTransport::new(header) }.unwrap();
        let input = VirtIOInput::<HalImpl, MmioTransport>::new(transport)
            .expect("failed to create input driver");
        Self::new(input, max_events)
    }
}

impl InputDevice for VirtIOInputDriver {
    fn is_empty(&self) -> bool {
        let inner = self.inner.lock();
        inner.events.is_empty()
    }

    fn read_event_with_block(&self) -> u64 {
        loop {
            {
                let mut inner = self.inner.lock();
                if let Some(event) = inner.events.pop_front() {
                    return event;
                }
                // let process = current_task().unwrap();
                // process.update_state(TaskState::Waiting);
                // inner.wait_queue.push_back(process.clone());
                let task = DRIVER_TASK.get().unwrap().get_task();
                task.to_wait();
                inner.wait_queue.push_back(task);
            } // drop the lock
            // schedule();
            DRIVER_TASK.get().unwrap().suspend();
        }
    }

    fn read_event_without_block(&self) -> Option<u64> {
        let mut inner = self.inner.lock();
        inner.events.pop_front()
    }
}

impl DeviceBase for VirtIOInputDriver {
    fn hand_irq(&self) {
        let mut inner = self.inner.lock();
        inner.driver.ack_interrupt();
        let mut count = 0;
        while let Some(event) = inner.driver.pop_pending_event() {
            let result =
                (event.event_type as u64) << 48 | (event.code as u64) << 32 | (event.value) as u64;
            info!("event: {:?}", event);
            if inner.events.len() >= inner.max_events as usize {
                // remove the first event
                inner.events.pop_front();
            }
            inner.events.push_back(result);
            count += 1;
        }
        while !inner.wait_queue.is_empty() && count > 0 {
            let task = inner.wait_queue.pop_front().unwrap();
            // process.update_state(TaskState::Ready);
            // GLOBAL_TASK_MANAGER.add_task(Arc::new(FifoTask::new(process)));
            task.to_wakeup();
            DRIVER_TASK.get().unwrap().put_task(task);
            count -= 1;
        }
        info!("read {} events", count);
    }
}
