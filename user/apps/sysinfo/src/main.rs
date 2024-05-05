#![no_main]
#![no_std]

extern crate Mstd;
extern crate alloc;

use alloc::{boxed::Box, rc::Rc, vec, vec::Vec};

use slint::{platform::WindowEvent, SharedString};
use slint_helper::{MyPlatform, SwapBuffer};
use virt2slint::Converter;
use Mstd::io::{keyboard_or_mouse_event, VIRTGPU_XRES, VIRTGPU_YRES};

slint::include_modules!();
#[no_mangle]
fn main() {
    let window = slint::platform::software_renderer::MinimalSoftwareWindow::new(Default::default());
    slint::platform::set_platform(Box::new(MyPlatform::new(window.clone()))).unwrap();

    window.set_size(slint::PhysicalSize::new(
        VIRTGPU_XRES as u32,
        VIRTGPU_YRES as u32,
    ));
    let sysinfo = SysInfo::new().unwrap();
    sysinfo.set_buffer_mem_size_kb(0);
    sysinfo.set_cpu_count("1".into());
    sysinfo.set_cpu_model("riscv64".into());
    sysinfo.set_cpu_vendor("qemu".into());
    sysinfo.set_mem_size_kb(1024 * 1024);
    sysinfo.set_os_name("Alien".into());
    sysinfo.set_uptime("2023.8.20".into());
    sysinfo.set_swap_free_kb(0);
    sysinfo.set_swap_used_kb(0);
    sysinfo.set_swap_total_kb(0);

    //  in-out property <[{devfs: string, mnt: string, total: int, free: int}]> partitions;
    let mount = Rc::new(
        slint::VecModel::<(SharedString, i32, SharedString, i32)>::from(vec![
            (
                "root".into(),
                120 * 1024 * 1024,
                "/".into(),
                128 * 1024 * 1024,
            ),
            (
                "tmpfs".into(),
                1024 * 1024 * 64,
                "/tmp".into(),
                64 * 1024 * 1024,
            ),
        ]),
    );
    sysinfo.set_partitions(mount.into());
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
            render.render(work_buffer, VIRTGPU_XRES);
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
