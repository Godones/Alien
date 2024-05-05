// Copyright © SixtyFPS GmbH <info@slint.devfs>
// SPDX-License-Identifier: MIT

#![no_std]
#![no_main]
extern crate Mstd;
extern crate alloc;

use alloc::{rc::Rc, vec::Vec};

use slint::{platform::WindowEvent, Model};
use slint_helper::{MyPlatform, SwapBuffer};
use virt2slint::Converter;
use Mstd::io::{keyboard_or_mouse_event, VIRTGPU_XRES, VIRTGPU_YRES};

#[no_mangle]
fn fmod(x: f64, y: f64) -> f64 {
    libm::fmod(x, y)
}

#[no_mangle]
fn fmodf(x: f32, y: f32) -> f32 {
    libm::fmodf(x, y)
}

slint::include_modules!();

struct PrinterQueueData {
    data: Rc<slint::VecModel<PrinterQueueItem>>,
    print_progress_timer: slint::Timer,
}

impl PrinterQueueData {
    fn push_job(&self, title: slint::SharedString) {
        self.data.push(PrinterQueueItem {
            status: JobStatus::Waiting,
            progress: 0,
            title,
            owner: env!("CARGO_PKG_AUTHORS").into(),
            pages: 1,
            size: "100kB".into(),
            submission_date: "".into(),
        })
    }
}

#[no_mangle]
fn main() -> ! {
    let window = slint::platform::software_renderer::MinimalSoftwareWindow::new(Default::default());
    slint::platform::set_platform(alloc::boxed::Box::new(MyPlatform::new(window.clone()))).unwrap();
    window.set_size(slint::PhysicalSize::new(
        VIRTGPU_XRES as u32,
        VIRTGPU_YRES as u32,
    ));

    let main_window = MainWindow::new().unwrap();
    main_window.set_ink_levels(
        [
            InkLevel {
                color: slint::Color::from_rgb_u8(0, 255, 255),
                level: 0.40,
            },
            InkLevel {
                color: slint::Color::from_rgb_u8(255, 0, 255),
                level: 0.20,
            },
            InkLevel {
                color: slint::Color::from_rgb_u8(255, 255, 0),
                level: 0.50,
            },
            InkLevel {
                color: slint::Color::from_rgb_u8(0, 0, 0),
                level: 0.80,
            },
        ]
        .into(),
    );

    let default_queue: Vec<PrinterQueueItem> = main_window
        .global::<PrinterQueue>()
        .get_printer_queue()
        .iter()
        .collect();
    let printer_queue = Rc::new(PrinterQueueData {
        data: Rc::new(slint::VecModel::from(default_queue.clone())),
        print_progress_timer: Default::default(),
    });
    main_window
        .global::<PrinterQueue>()
        .set_printer_queue(printer_queue.data.clone().into());

    main_window.on_quit(move || {
        slint::quit_event_loop().unwrap();
    });

    let printer_queue_copy = printer_queue.clone();
    main_window
        .global::<PrinterQueue>()
        .on_start_job(move |title| {
            printer_queue_copy.push_job(title);
        });

    let printer_queue_copy = printer_queue.clone();
    main_window
        .global::<PrinterQueue>()
        .on_cancel_job(move |idx| {
            printer_queue_copy.data.remove(idx as usize);
        });

    let printer_queue_weak = Rc::downgrade(&printer_queue);
    printer_queue.print_progress_timer.start(
        slint::TimerMode::Repeated,
        core::time::Duration::from_secs(1),
        move || {
            if let Some(printer_queue) = printer_queue_weak.upgrade() {
                if printer_queue.data.row_count() > 0 {
                    let mut top_item = printer_queue.data.row_data(0).unwrap();
                    top_item.progress += 1;
                    top_item.status = JobStatus::Printing;
                    if top_item.progress > 100 {
                        printer_queue.data.remove(0);
                        if printer_queue.data.row_count() == 0 {
                            return;
                        }
                        top_item = printer_queue.data.row_data(0).unwrap();
                    }
                    printer_queue.data.set_row_data(0, top_item);
                } else {
                    printer_queue.data.set_vec(default_queue.clone());
                }
            }
        },
    );

    // main_window.run().unwrap();
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
