#![no_std]
#![no_main]
extern crate alloc;

use alloc::boxed::Box;
use alloc::rc::Rc;
use alloc::vec;
use alloc::vec::Vec;
use core::clone::Clone;
use core::convert::Into;
use core::default::Default;
use core::iter::Iterator;
use slint::platform::WindowEvent;
use slint::{FilterModel, Model, SortModel};
use virt2slint::Converter;
use slint_helper::{MyPlatform, SwapBuffer};
use Mstd::io::{keyboard_or_mouse_event, VIRTGPU_XRES, VIRTGPU_YRES};

slint::include_modules!();

fn create_app() -> MainWindow {
    let main_window = MainWindow::new().unwrap();
    let todo_model = Rc::new(slint::VecModel::<TodoItem>::from(vec![
        TodoItem {
            checked: true,
            title: "Implement the .slint file".into(),
        },
        TodoItem {
            checked: true,
            title: "Do the Rust part".into(),
        },
        TodoItem {
            checked: false,
            title: "Make the C++ code".into(),
        },
        TodoItem {
            checked: false,
            title: "Write some JavaScript code".into(),
        },
        TodoItem {
            checked: false,
            title: "Test the application".into(),
        },
        TodoItem {
            checked: false,
            title: "Ship to customer".into(),
        },
        TodoItem {
            checked: false,
            title: "???".into(),
        },
        TodoItem {
            checked: false,
            title: "Profit".into(),
        },
    ]));

    main_window.on_todo_added({
        let todo_model = todo_model.clone();
        move |text| {
            todo_model.push(TodoItem {
                checked: false,
                title: text,
            })
        }
    });
    main_window.on_remove_done({
        let todo_model = todo_model.clone();
        move || {
            let mut offset = 0;
            for i in 0..todo_model.row_count() {
                if todo_model.row_data(i - offset).unwrap().checked {
                    todo_model.remove(i - offset);
                    offset += 1;
                }
            }
        }
    });

    let weak_window = main_window.as_weak();
    main_window.on_popup_confirmed(move || {
        let window = weak_window.unwrap();
        window.hide().unwrap();
    });

    {
        let weak_window = main_window.as_weak();
        let todo_model = todo_model.clone();
        main_window.window().on_close_requested(move || {
            let window = weak_window.unwrap();

            if todo_model.iter().any(|t| !t.checked) {
                window.invoke_show_confirm_popup();
                slint::CloseRequestResponse::KeepWindowShown
            } else {
                slint::CloseRequestResponse::HideWindow
            }
        });
    }

    main_window.on_apply_sorting_and_filtering({
        let weak_window = main_window.as_weak();
        let todo_model = todo_model.clone();

        move || {
            let window = weak_window.unwrap();
            window.set_todo_model(todo_model.clone().into());

            if window.get_hide_done_items() {
                window.set_todo_model(
                    Rc::new(FilterModel::new(window.get_todo_model(), |e| !e.checked)).into(),
                );
            }

            if window.get_is_sort_by_name() {
                window.set_todo_model(
                    Rc::new(SortModel::new(window.get_todo_model(), |lhs, rhs| {
                        lhs.title.to_lowercase().cmp(&rhs.title.to_lowercase())
                    }))
                    .into(),
                );
            }
        }
    });

    main_window.set_show_header(true);
    main_window.set_todo_model(todo_model.into());
    main_window
}

#[no_mangle]
fn main() {
    let window = slint::platform::software_renderer::MinimalSoftwareWindow::new(Default::default());
    slint::platform::set_platform(Box::new(MyPlatform::new(window.clone()))).unwrap();

    window.set_size(slint::PhysicalSize::new(
        VIRTGPU_XRES as u32,
        VIRTGPU_YRES as u32,
    ));

    let _ui = create_app();

    let mut swap_buffer = SwapBuffer::new();
    let mut x = 0;
    let mut y = 0;
    let mut converter = Converter::new(32767, VIRTGPU_XRES as isize, VIRTGPU_YRES as isize);
    loop {
        // Let Slint run the timer hooks and update animations.
        slint::platform::update_timers_and_animations();
        let events = checkout_event(&mut converter,&mut x, &mut y);
        events.iter().for_each(|event| {
            window.dispatch_event(event.clone());
        });
        window.draw_if_needed(|render| {
            let work_buffer = swap_buffer.work_buffer();
            // Do the rendering!
            render.render(work_buffer, VIRTGPU_XRES);
            // tell the screen driver to display the other buffer.
            swap_buffer.swap_buffer();
        });
    }
}

fn checkout_event(converter: &mut Converter,x: &mut isize, y: &mut isize) -> Vec<WindowEvent> {
    let mut events = [0; 100];
    let event_num = keyboard_or_mouse_event(&mut events);
    let mut res = Vec::new();
    for i in 0..event_num as usize {
        let event = events[i];
        // let window_event = input2event(event, x, y);
        let window_event = converter.convert(event, x, y);
        window_event.map(|e|{
            res.push(e);
        });
    }
    res
}
