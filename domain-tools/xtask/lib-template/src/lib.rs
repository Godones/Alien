#![no_std]
#![forbid(unsafe_code)]
extern crate alloc;
use alloc::boxed::Box;
use interface::INTERFACE;

pub fn main()->Box<dyn INTERFACE>{

}
