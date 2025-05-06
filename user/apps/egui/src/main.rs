#![no_main]
#![no_std]
extern crate Mstd;
extern crate alloc;
use Mstd::{
    io::{flush_frame_buffer, frame_buffer, VIRTGPU_XRES, VIRTGPU_YRES},
    println,
};

struct GpuDevice {
    frame_buffer: &'static mut [u8],
}

impl GpuDevice {
    pub fn new() -> Self {
        let fb = frame_buffer();
        Self { frame_buffer: fb }
    }
}

impl GpuDevice {
    fn flush(&self) {
        flush_frame_buffer();
    }

    fn draw_point(&mut self, x: i32, y: i32, color: u32) {
        let fb = &mut self.frame_buffer;
        let offset = (y * VIRTGPU_XRES as i32 + x) as usize * 4;
        fb[offset] = (color >> 16) as u8;
        fb[offset + 1] = (color >> 8) as u8;
        fb[offset + 2] = color as u8;
        fb[offset + 3] = 0xff;
    }
}

#[no_mangle]
fn main() {
    println!("embedded graphics demo");
    let mut device = GpuDevice::new();
    for x in 0..VIRTGPU_XRES {
        for y in 0..VIRTGPU_YRES {
            device.draw_point(x as i32, y as i32, 0x00ff00);
        }
    }
    device.flush();
    println!("embedded graphics demo end");
}
