#![no_std]

extern crate alloc;

pub use buffer::SwapBuffer;
pub use platform::MyPlatform;

mod buffer;
mod platform;
