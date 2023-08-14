#![no_main]
#![no_std]

extern crate Mstd;
extern crate alloc;
extern crate libm;

use alloc::rc::Rc;
use alloc::vec::Vec;
use core::time::Duration;

use rand::prelude::{SliceRandom, SmallRng};
use rand::SeedableRng;
use slint::platform::software_renderer::Rgb565Pixel;
use slint::platform::WindowEvent;
use slint::{Model, Rgb8Pixel, VecModel};

use input2event::input2event;
use Mstd::gui::embedded_graphics::pixelcolor::raw::RawU16;
use Mstd::gui::embedded_graphics::pixelcolor::Rgb565;
use Mstd::gui::embedded_graphics::prelude::*;
use Mstd::gui::embedded_graphics::primitives::Rectangle;
use Mstd::io::{
    flush_frame_buffer, frame_buffer, keyboard_or_mouse_event, VIRTGPU_XRES, VIRTGPU_YRES,
};
use Mstd::time::{get_time_ms, TimeSpec, TimeVal};

slint::include_modules!();

#[no_mangle]
fn fmod(x: f64, y: f64) -> f64 {
    libm::fmod(x, y)
}

pub fn time_spec() -> TimeSpec {
    let time = TimeVal::now();
    TimeSpec::from(time)
}

struct MyPlatform {
    window: Rc<slint::platform::software_renderer::MinimalSoftwareWindow>,
    start_timer: TimeSpec,
}

impl slint::platform::Platform for MyPlatform {
    fn create_window_adapter(
        &self,
    ) -> Result<Rc<dyn slint::platform::WindowAdapter>, slint::PlatformError> {
        Ok(self.window.clone())
    }
    fn duration_since_start(&self) -> Duration {
        let old_time = self.start_timer;
        let new_time = time_spec();
        Duration::new(new_time.tv_sec as u64, new_time.tv_nsec as u32)
            - Duration::new(old_time.tv_sec as u64, old_time.tv_nsec as u32)
    }
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
    slint::platform::set_platform(alloc::boxed::Box::new(MyPlatform {
        window: window.clone(),
        start_timer: time_spec(),
    }))
    .unwrap();

    let _ui = create_slint_app();
    window.set_size(slint::PhysicalSize::new(
        VIRTGPU_XRES as u32,
        VIRTGPU_YRES as u32,
    ));
    // let mut line_buffer = [Rgb565Pixel(0); VIRTGPU_XRES];
    // let mut display = Display::new(Size::new(1280, 800), Point::new(0, 0));
    let mut buffer2: Vec<Rgb8Pixel> = Vec::new();
    buffer2.reserve_exact(1280 * 800);
    unsafe {
        buffer2.set_len(1280 * 800);
    }
    // let mut buf1 = frame_buffer();
    let mut buffer1: Vec<Rgb8Pixel> = Vec::new();
    buffer1.reserve_exact(1280 * 800);
    unsafe {
        buffer1.set_len(1280 * 800);
    }
    let mut currently_displayed_buffer: &mut [_] = buffer1.as_mut_slice();
    let mut work_buffer: &mut [_] = buffer2.as_mut_slice();
    let display_buf = frame_buffer();
    let mut swap_buffers = |buf: &[Rgb8Pixel]| {
        assert_eq!(buf.len(), 1280 * 800);
        buf.iter().enumerate().for_each(|(i, p)| {
            let idx = i * 4;
            display_buf[idx] = p.b;
            display_buf[idx + 1] = p.g;
            display_buf[idx + 2] = p.r;
        });
        flush_frame_buffer();
    };

    let mut x = 0;
    let mut y = 0;
    loop {
        // Let Slint run the timer hooks and update animations.
        slint::platform::update_timers_and_animations();
        let event = checkout_event(&mut x, &mut y);
        // println!("event: {:?}", event);
        if let Some(event) = event {
            window.dispatch_event(event);
        }
        window.draw_if_needed(|render| {
            render.render(work_buffer, 1280);

            // tell the screen driver to display the other buffer.
            swap_buffers(work_buffer);

            // Swap the buffer references for our next iteration
            // (this just swap the reference, not the actual data)
            core::mem::swap::<&mut [_]>(&mut work_buffer, &mut currently_displayed_buffer);
        });
    }
}

static mut EVENTS: &'static mut [u64] = &mut [0; 100];
static mut COUNT: usize = 0;
static mut LAST_READ: usize = 0;

fn checkout_event(x: &mut i32, y: &mut i32) -> Option<WindowEvent> {
    unsafe {
        if COUNT == 0 {
            COUNT = keyboard_or_mouse_event(&mut EVENTS) as usize;
            LAST_READ = 0;
        }
        return if COUNT > 0 {
            let event = EVENTS[LAST_READ];
            LAST_READ += 1;
            COUNT -= 1;
            let window_event = input2event(event, x, y).unwrap();
            Some(window_event)
        } else {
            None
        };
    }
}

struct DisplayWrapper<'a, T> {
    display: &'a mut T,
    line_buffer: &'a mut [Rgb565Pixel],
}

impl<T: DrawTarget<Color = Rgb565>> slint::platform::software_renderer::LineBufferProvider
    for DisplayWrapper<'_, T>
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
        self.display
            .fill_contiguous(
                &Rectangle::new(
                    Point::new(range.start as _, line as _),
                    Size::new(range.len() as _, 1),
                ),
                self.line_buffer[range.clone()].iter().map(|p| {
                    let raw = RawU16::new(p.0);
                    raw.into()
                }),
            )
            .map_err(drop)
            .unwrap();
    }
}
