#![no_main]
#![no_std]
extern crate Mstd;
extern crate alloc;

use alloc::sync::Arc;

use simplegui::{
    complex::{desktop::Desktop, terminal::GodTerminal},
    GPUDevice,
};
use Mstd::{
    gui::embedded_graphics::geometry::{Point, Size},
    io::{flush_frame_buffer, frame_buffer, VIRTGPU_XRES, VIRTGPU_YRES},
    println,
    sync::mutex::Mutex,
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

impl GPUDevice for GpuDevice {
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
    simplegui::init_gpu(Arc::new(Mutex::new(GpuDevice::new())));
    let desk = Desktop::new(VIRTGPU_XRES as u32, VIRTGPU_YRES as u32);
    desk.paint();
    let terminal = GodTerminal::new(Size::new(300, 300), Point::new(100, 100));
    terminal.add_str("hello world");
}
