#![no_main]
#![no_std]

extern crate Mstd;
extern crate alloc;

use alloc::{boxed::Box, vec::Vec};

use slint::platform::WindowEvent;
use slint_helper::{MyPlatform, SwapBuffer};
use virt2slint::Converter;
use Mstd::io::{keyboard_or_mouse_event, VIRTGPU_XRES, VIRTGPU_YRES};

slint::include_modules!();

fn create_slint_app() -> AppWindow {
    let ui = AppWindow::new().expect("Failed to load UI");

    let ui_handle = ui.as_weak();
    ui.on_request_increase_value(move || {
        let ui = ui_handle.unwrap();
        ui.set_counter(ui.get_counter() + 1);
    });
    ui
}

#[no_mangle]
fn main() {
    let window = slint::platform::software_renderer::MinimalSoftwareWindow::new(Default::default());
    slint::platform::set_platform(Box::new(MyPlatform::new(window.clone()))).unwrap();
    let _ui = create_slint_app();
    window.set_size(slint::PhysicalSize::new(
        VIRTGPU_XRES as u32,
        VIRTGPU_YRES as u32,
    ));
    let mut swap_buffer = SwapBuffer::new();
    let mut x = 0;
    let mut y = 0;
    let mut converter = Converter::new(32767, VIRTGPU_XRES as isize, VIRTGPU_YRES as isize);
    loop {
        // Let Slint run the timer hooks and update animations.
        slint::platform::update_timers_and_animations();
        let events = checkout_event(&mut converter, &mut x, &mut y);
        events.iter().for_each(|event| {
            window.dispatch_event(event.clone());
        });
        window.draw_if_needed(|render| {
            let work_buffer = swap_buffer.work_buffer();
            // Do the rendering!
            render.render(work_buffer, 1280);
            // tell the screen driver to display the other buffer.
            swap_buffer.swap_buffer();
        });
    }
}

fn checkout_event(converter: &mut Converter, x: &mut isize, y: &mut isize) -> Vec<WindowEvent> {
    let mut events = [0; 100];
    let event_num = keyboard_or_mouse_event(&mut events);
    let mut res = Vec::new();
    for i in 0..event_num as usize {
        let event = events[i];
        // let window_event = input2event(event, x, y);
        let window_event = converter.convert(event, x, y);
        window_event.map(|e| {
            res.push(e);
        });
    }
    res
}
