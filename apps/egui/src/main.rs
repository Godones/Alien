#![no_main]
#![no_std]
extern crate Mstd;
extern crate alloc;

use simplegui::complex::desktop::Desktop;
use simplegui::complex::terminal::GodTerminal;
use Mstd::gui::embedded_graphics::geometry::{Point, Size};
use Mstd::io::{flush_frame_buffer, frame_buffer, VIRTGPU_XRES, VIRTGPU_YRES};
use Mstd::println;
use Mstd::sync::mutex::{Mutex, Once};

struct GpuDevice {
    frame_buffer: &'static mut [u8],
}

impl GpuDevice {
    pub fn new() -> Self {
        let fb = frame_buffer();
        Self { frame_buffer: fb }
    }
    pub fn buffer(&mut self) -> &mut [u8] {
        self.frame_buffer
    }
}

static GPU: Once<Mutex<GpuDevice>> = Once::new();

#[no_mangle]
fn __draw_point(x: i32, y: i32, color: u32) {
    let mut gpu = GPU.get().unwrap().lock();
    let fb = gpu.buffer();
    let offset = (y * VIRTGPU_XRES as i32 + x) as usize * 4;
    fb[offset] = (color >> 16) as u8;
    fb[offset + 1] = (color >> 8) as u8;
    fb[offset + 2] = color as u8;
    fb[offset + 3] = 0xff;
}

#[no_mangle]
fn __gpu_flush() {
    flush_frame_buffer();
}

#[no_mangle]
fn main() {
    println!("embedded graphics demo");
    GPU.call_once(|| Mutex::new(GpuDevice::new()));
    let desk = Desktop::new(VIRTGPU_XRES as u32, VIRTGPU_YRES as u32);
    desk.paint();
    let terminal = GodTerminal::new(Size::new(300, 300), Point::new(100, 100));
    terminal.add_str("hello world");
}
