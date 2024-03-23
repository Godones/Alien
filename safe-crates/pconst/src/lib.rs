#![feature(maybe_uninit_uninit_array)]
#![feature(maybe_uninit_slice)]
#![feature(maybe_uninit_write_slice)]
#![feature(ip)]
#![no_std]
#![allow(unused)]
extern crate alloc;

pub use consts::*;

pub mod aux;
mod consts;
pub mod io;
pub mod ipc;
pub mod net;
pub mod signal;
pub mod sys;
pub mod task;
pub mod time;
