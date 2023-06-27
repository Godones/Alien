use crate::io::{flush_frame_buffer, frame_buffer, VIRTGPU_XRES};

use self::embedded_graphics::pixelcolor::Rgb565;
use self::embedded_graphics::prelude::*;

pub mod embedded_graphics {
    pub use embedded_graphics::*;
}

pub struct Display {
    pub size: Size,
    pub point: Point,
    //pub fb: Arc<&'static mut [u8]>,
    pub fb: &'static mut [u8],
}

impl Display {
    pub fn new(size: Size, point: Point) -> Self {
        let fb = frame_buffer();
        Self { size, point, fb }
    }
}

impl OriginDimensions for Display {
    fn size(&self) -> Size {
        self.size
    }
}

impl DrawTarget for Display {
    type Color = Rgb565;

    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
        where
            I: IntoIterator<Item=embedded_graphics::Pixel<Self::Color>>,
    {
        pixels.into_iter().for_each(|px| {
            let idx = ((self.point.y + px.0.y) * VIRTGPU_XRES as i32 + self.point.x + px.0.x)
                as usize
                * 4;
            if idx + 2 >= self.fb.len() {
                return;
            }
            self.fb[idx] = px.1.b();
            self.fb[idx + 1] = px.1.g();
            self.fb[idx + 2] = px.1.r();
        });
        flush_frame_buffer();
        Ok(())
    }
}