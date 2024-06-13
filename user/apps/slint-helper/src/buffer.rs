use alloc::vec::Vec;

use slint::Rgb8Pixel;
use Mstd::io::{flush_frame_buffer, frame_buffer};

pub struct SwapBuffer {
    display_buf: &'static mut [u8],
    buf1: Vec<Rgb8Pixel>,
    buf2: Vec<Rgb8Pixel>,
    choice: bool,
}

impl SwapBuffer {
    pub fn new() -> Self {
        let mut buffer2: Vec<Rgb8Pixel> = Vec::new();
        buffer2.reserve_exact(1280 * 800);
        unsafe {
            buffer2.set_len(1280 * 800);
        }
        // let mut buf1 = frame_buffer();
        let mut buffer1: Vec<Rgb8Pixel> = Vec::new();
        buffer1.reserve_exact(1280 * 800);
        unsafe {
            buffer1.set_len(1280 * 800);
        }
        let display_buf = frame_buffer();
        SwapBuffer {
            display_buf,
            buf1: buffer1,
            buf2: buffer2,
            choice: false,
        }
    }
    pub fn work_buffer(&mut self) -> &mut [Rgb8Pixel] {
        if self.choice {
            &mut self.buf1
        } else {
            &mut self.buf2
        }
    }

    pub fn swap_buffer(&mut self) {
        let buf = if self.choice {
            self.buf1.as_slice()
        } else {
            self.buf2.as_slice()
        };
        buf.iter().enumerate().for_each(|(i, p)| {
            let idx = i * 4;
            self.display_buf[idx] = p.b;
            self.display_buf[idx + 1] = p.g;
            self.display_buf[idx + 2] = p.r;
        });
        flush_frame_buffer();
        self.choice = !self.choice;
    }
}
