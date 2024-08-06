#![no_main]
#![no_std]

extern crate Mstd;
extern crate alloc;
extern crate libm;

use alloc::{rc::Rc, vec::Vec};
use core::time::Duration;

use rand::{
    prelude::{SliceRandom, SmallRng},
    SeedableRng,
};
use slint::{platform::WindowEvent, Model, VecModel};
use slint_helper::{MyPlatform, SwapBuffer};
use virt2slint::Converter;
use Mstd::{
    io::{keyboard_or_mouse_event, VIRTGPU_XRES, VIRTGPU_YRES},
    println,
    process::exit,
    time::get_time_ms,
};

slint::include_modules!();

#[no_mangle]
fn fmod(x: f64, y: f64) -> f64 {
    libm::fmod(x, y)
}

fn create_slint_app() -> MainWindow {
    let main_window = MainWindow::new().expect("Failed to load UI");

    let mut tiles: Vec<TileData> = main_window.get_memory_tiles().iter().collect();
    tiles.extend(tiles.clone());
    let sed = get_time_ms();
    let mut small_rng = SmallRng::seed_from_u64(sed as u64);
    // let rand_num = small_rng.next_u64();
    tiles.shuffle(&mut small_rng);
    let tiles_model = Rc::new(VecModel::from(tiles));
    main_window.set_memory_tiles(tiles_model.clone().into());
    let main_window_weak = main_window.as_weak();
    main_window.on_check_if_pair_solved(move || {
        let mut flipped_tiles = tiles_model
            .iter()
            .enumerate()
            .filter(|(_, tile)| tile.image_visible && !tile.solved);

        if let (Some((t1_idx, mut t1)), Some((t2_idx, mut t2))) =
            (flipped_tiles.next(), flipped_tiles.next())
        {
            let is_pair_solved = t1 == t2;
            if is_pair_solved {
                t1.solved = true;
                tiles_model.set_row_data(t1_idx, t1);
                t2.solved = true;
                tiles_model.set_row_data(t2_idx, t2);
            } else {
                main_window_weak.unwrap().set_disable_tiles(true);
                let main_window_weak = main_window_weak.clone();
                let tiles_model = tiles_model.clone();
                slint::Timer::single_shot(Duration::from_secs(1), move || {
                    main_window_weak.unwrap().set_disable_tiles(false);
                    t1.image_visible = false;
                    tiles_model.set_row_data(t1_idx, t1);
                    t2.image_visible = false;
                    tiles_model.set_row_data(t2_idx, t2);
                })
            }
        }
    });
    main_window
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
    let _ui = create_slint_app();
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
        let e = InputEvent::from(event);
        if e.event_type == 1 && (e.code == 1 || e.code == 273) && e.value == 0 {
            println!("ESC or right-click pressed, exit");
            exit(0);
        }
        let window_event = converter.convert(event, x, y);
        window_event.map(|e| {
            res.push(e);
        });
    }
    res
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct InputEvent {
    /// Event type.
    pub event_type: u16,
    /// Event code.
    pub code: u16,
    /// Event value.
    pub value: u32,
}

impl From<u64> for InputEvent {
    fn from(event: u64) -> Self {
        let event_type = (event >> 48) as u16;
        let code = ((event >> 32) & 0xFFFF) as u16;
        let value = (event & 0xFFFFFFFF) as u32;
        Self {
            event_type,
            code,
            value,
        }
    }
}
