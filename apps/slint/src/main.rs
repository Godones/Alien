#![no_main]
#![no_std]

#[macro_use]
extern crate Mstd;
extern crate alloc;

use alloc::rc::Rc;

use embedded_graphics::pixelcolor::raw::RawU16;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::Rectangle;
use slint::platform::software_renderer::Rgb565Pixel;
use slint::platform::WindowEvent;

use Mstd::io::{flush_frame_buffer, frame_buffer, VIRTGPU_XRES, VIRTGPU_YRES};
use Mstd::time::{sleep, TimeSpec, TimeVal};

slint::include_modules!();


pub struct Timer;

impl Timer {
    pub fn time_spec(&self) -> TimeSpec {
        let time = TimeVal::now();
        TimeSpec::from(time)
    }
}


struct MyPlatform {
    window: Rc<slint::platform::software_renderer::MinimalSoftwareWindow>,
    timer: Timer,
}

impl slint::platform::Platform for MyPlatform {
    fn create_window_adapter(&self) -> Result<Rc<dyn slint::platform::WindowAdapter>, slint::PlatformError> {
        Ok(self.window.clone())
    }
    fn duration_since_start(&self) -> core::time::Duration {
        let time_spec = self.timer.time_spec();
        core::time::Duration::new(time_spec.tv_sec as u64, time_spec.tv_nsec as u32)
    }
}


fn create_slint_app() -> AppWindow {
    let ui = AppWindow::new().expect("Failed to load UI");

    let ui_handle = ui.as_weak();
    ui.on_request_increase_value(move || {
        let ui = ui_handle.unwrap();
        ui.set_counter(ui.get_counter() + 1);
    });
    ui
}


struct DisplayWrapper<'a, T> {
    display: &'a mut T,
    line_buffer: &'a mut [Rgb565Pixel],
}

impl<T: DrawTarget<Color=Rgb565>>
slint::platform::software_renderer::LineBufferProvider for DisplayWrapper<'_, T>
{
    type TargetPixel = Rgb565Pixel;
    fn process_line(
        &mut self,
        line: usize,
        range: core::ops::Range<usize>,
        render_fn: impl FnOnce(&mut [Self::TargetPixel]),
    ) {
        // Render into the line
        render_fn(&mut self.line_buffer[range.clone()]);

        // Send the line to the screen using DrawTarget::fill_contiguous
        self.display.fill_contiguous(
            &Rectangle::new(Point::new(range.start as _, line as _), Size::new(range.len() as _, 1)),
            self.line_buffer[range.clone()].iter().map(|p| {
                let raw = RawU16::new(p.0);
                raw.into()
            }),
        ).map_err(drop).unwrap();
    }
}


#[no_mangle]
fn main() {
    let timer = Timer;
    let window = slint::platform::software_renderer::MinimalSoftwareWindow::new(Default::default());
    slint::platform::set_platform(alloc::boxed::Box::new(MyPlatform {
        window: window.clone(),
        timer,
    })).unwrap();

    let _ui = create_slint_app();
    window.set_size(slint::PhysicalSize::new(VIRTGPU_XRES as u32, VIRTGPU_YRES as u32));

    let mut line_buffer = [Rgb565Pixel(0); VIRTGPU_XRES];

    let mut display = Display::new(Size::new(1280, 800), Point::new(0, 0));
    loop {
        // Let Slint run the timer hooks and update animations.
        slint::platform::update_timers_and_animations();
        // window.dispatch_event()
        window.draw_if_needed(|render| {
            let display_wrapper = DisplayWrapper {
                display: &mut display,
                line_buffer: &mut line_buffer,
            };
            render.render_by_line(display_wrapper);
        });
        if !window.has_active_animations() {
            if let Some(duration) = slint::platform::duration_until_next_timer_update() {
                // ... schedule a timer interrupt in `duration` ...
                let ms = duration.as_millis() as usize;
                sleep(ms)
            }
        }
    }
}


fn checkout_event() -> WindowEvent {
    WindowEvent::KeyPressed {
        text: Default::default(),
    }
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