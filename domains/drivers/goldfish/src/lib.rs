#![no_std]
#![deny(unsafe_code)]
#![allow(unused)]
mod rtc;

extern crate alloc;
extern crate malloc;

use crate::rtc::GoldFishRtc;
use alloc::sync::Arc;
use interface::{Basic, RtcDomain, RtcTime};
use libsyscall::println;
use region::SafeIORegion;
use rref::{RRef, RpcResult};
use time::macros::offset;
use time::OffsetDateTime;

impl Basic for GoldFishRtc {}

impl RtcDomain for GoldFishRtc {
    fn read_time(&self, mut time: RRef<RtcTime>) -> RpcResult<RRef<RtcTime>> {
        let time_stamp = self.read_raw_time();
        let t = self.read_time_fmt();
        *time = t;
        Ok(time)
    }

    fn handle_irq(&self) -> RpcResult<()> {
        unimplemented!()
    }
}

pub fn main(safe_region: SafeIORegion) -> Arc<dyn RtcDomain> {
    let rtc = Arc::new(GoldFishRtc::new(safe_region));
    println!("current time: {:?}", rtc);
    rtc
}
