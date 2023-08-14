#![no_main]
#![no_std]

extern crate Mstd;
extern crate alloc;

use alloc::vec::Vec;

use slint::platform::WindowEvent;

use input2event::input2event;
use slint_helper::{MyPlatform, SwapBuffer};
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
    slint::platform::set_platform(alloc::boxed::Box::new(MyPlatform::new(window.clone()))).unwrap();
    let _ui = create_slint_app();
    window.set_size(slint::PhysicalSize::new(
        VIRTGPU_XRES as u32,
        VIRTGPU_YRES as u32,
    ));
    let mut swap_buffer = SwapBuffer::new();
    let mut x = 0;
    let mut y = 0;
    loop {
        // Let Slint run the timer hooks and update animations.
        slint::platform::update_timers_and_animations();
        let events = checkout_event(&mut x, &mut y);
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
