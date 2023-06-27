#![no_main]
#![no_std]

extern crate Mstd;
extern crate alloc;

use alloc::rc::Rc;
use alloc::vec::Vec;

use slint::platform::software_renderer::Rgb565Pixel;
use slint::platform::WindowEvent;

use input2event::input2event;
use Mstd::gui::Display;
use Mstd::gui::embedded_graphics::pixelcolor::raw::RawU16;
use Mstd::gui::embedded_graphics::pixelcolor::Rgb565;
use Mstd::gui::embedded_graphics::prelude::*;
use Mstd::gui::embedded_graphics::primitives::Rectangle;
use Mstd::io::{keyboard_or_mouse_event, VIRTGPU_XRES, VIRTGPU_YRES};
use Mstd::time::{TimeSpec, TimeVal};

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
    let mut x = 0;
    let mut y = 0;
    loop {
        // Let Slint run the timer hooks and update animations.
        slint::platform::update_timers_and_animations();
        // window.dispatch_event()
        let events = checkout_event(&mut x, &mut y);
        events.iter().for_each(|event| {
            window.dispatch_event(event.clone());
        });
        window.draw_if_needed(|render| {
            let display_wrapper = DisplayWrapper {
                display: &mut display,
                line_buffer: &mut line_buffer,
            };
            render.render_by_line(display_wrapper);
        });
    }
}


fn checkout_event(x: &mut i32, y: &mut i32) -> Vec<WindowEvent> {
    let mut events = [0; 100];
    let event_num = keyboard_or_mouse_event(&mut events);
    let mut res = Vec::new();
    for i in 0..event_num as usize {
        let event = events[i];
        let window_event = input2event(event, x, y).unwrap();
        res.push(window_event);
    }
    res
}
