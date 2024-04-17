#![no_std]
#![forbid(unsafe_code)]
extern crate alloc;
use alloc::{boxed::Box, collections::VecDeque, sync::Arc};

use basic::{config::MAX_INPUT_EVENT_NUM, println, sync::Mutex};
use constants::{AlienError, AlienResult};
use interface::{Basic, BufInputDomain, DeviceBase, DomainType, InputDomain, SchedulerDomain};
use spin::Once;

static INPUT_DOMAIN: Once<Arc<dyn InputDomain>> = Once::new();
static SCHEDULER_DOMAIN: Once<Arc<dyn SchedulerDomain>> = Once::new();

#[derive(Debug)]
struct BufInput {
    max_events: u32,
    inner: Mutex<BufInputInner>,
}

#[derive(Debug)]
struct BufInputInner {
    events: VecDeque<u64>,
    wait_queue: VecDeque<usize>,
}

impl BufInput {
    pub fn new(max_events: u32) -> Self {
        let inner = BufInputInner {
            events: VecDeque::with_capacity(max_events as usize),
            wait_queue: VecDeque::new(),
        };
        BufInput {
            max_events,
            inner: Mutex::new(inner),
        }
    }
}

impl DeviceBase for BufInput {
    fn handle_irq(&self) -> AlienResult<()> {
        let mut inner = self.inner.lock();
        let input_domain = INPUT_DOMAIN.get().unwrap();
        let mut count = 0;
        while let Some(event) = input_domain.event_nonblock()? {
            // info!("event: {:?}", event);
            if inner.events.len() >= self.max_events as usize {
                // remove the first event
                inner.events.pop_front();
            }
            inner.events.push_back(event);
            count += 1;
        }
        while !inner.wait_queue.is_empty() && count > 0 {
            let tid = inner.wait_queue.pop_front().unwrap();
            SCHEDULER_DOMAIN
                .get()
                .unwrap()
                .wake_up_wait_task(tid)
                .unwrap();
            count -= 1;
        }
        // info!("read {} events", count);
        Ok(())
    }
}

impl Basic for BufInput {}

impl BufInputDomain for BufInput {
    fn init(&self, input_domain_name: &str) -> AlienResult<()> {
        let input_domain = basic::get_domain(input_domain_name).unwrap();
        match input_domain {
            DomainType::InputDomain(input) => {
                INPUT_DOMAIN.call_once(|| input);
                Ok(())
            }
            ty => {
                println!(
                    "find input domain by name {},ty: {:?}",
                    input_domain_name, ty
                );
                Err(AlienError::EINVAL)
            }
        }?;
        let scheduler_domain = basic::get_domain("scheduler").unwrap();
        match scheduler_domain {
            DomainType::SchedulerDomain(scheduler_domain) => {
                SCHEDULER_DOMAIN.call_once(|| scheduler_domain);
                Ok(())
            }
            _ => return Err(AlienError::EINVAL),
        }
    }

    fn event_block(&self) -> AlienResult<u64> {
        loop {
            let mut inner = self.inner.lock();
            if let Some(event) = inner.events.pop_front() {
                return Ok(event);
            }
            let scheduler = SCHEDULER_DOMAIN.get().unwrap();
            let tid = scheduler.current_tid()?.unwrap();
            inner.wait_queue.push_back(tid);
            drop(inner);
            scheduler.current_to_wait()?;
        }
    }

    fn event_nonblock(&self) -> AlienResult<Option<u64>> {
        let mut inner = self.inner.lock();
        Ok(inner.events.pop_front())
    }

    fn have_event(&self) -> AlienResult<bool> {
        let inner = self.inner.lock();
        Ok(!inner.events.is_empty())
    }
}

pub fn main() -> Box<dyn BufInputDomain> {
    Box::new(BufInput::new(MAX_INPUT_EVENT_NUM))
}
