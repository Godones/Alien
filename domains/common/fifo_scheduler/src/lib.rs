#![no_std]
#![forbid(unsafe_code)]

mod scheduler;

extern crate alloc;

use alloc::boxed::Box;

use common_scheduler::CommonSchedulerDomain;
use interface::SchedulerDomain;

use crate::scheduler::FiFoScheduler;

pub fn main() -> Box<dyn SchedulerDomain> {
    Box::new(CommonSchedulerDomain::new(Box::new(FiFoScheduler::new())))
}
