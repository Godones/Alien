use config::FRAME_SIZE;
use ksync::Mutex;
use platform::println;
use ptable::PhysPage;

use crate::frame::FrameTracker;

pub static INITRD_DATA: Mutex<Option<InitrdData>> = Mutex::new(None);

pub struct InitrdData {
    frames: FrameTracker,
    size: usize,
}

impl InitrdData {
    pub fn as_slice(&self) -> &[u8] {
        self.frames.as_bytes()[..self.size].as_ref()
    }
}

pub(super) fn relocate_removable_data() {
    let info = platform::platform_machine_info();
    if info.initrd.is_some() {
        let start = info.initrd.as_ref().unwrap().start;
        let end = info.initrd.as_ref().unwrap().end;
        let size = end - start;
        let np = (size + FRAME_SIZE - 1) / FRAME_SIZE;
        let frame_tracker = crate::alloc_frame_trackers(np);
        // copy data
        unsafe {
            core::ptr::copy_nonoverlapping(
                start as *const u8,
                frame_tracker.phys_addr().as_usize() as _,
                size,
            );
        }
        println!(
            "Relocate initrd data to {:#x}",
            frame_tracker.phys_addr().as_usize()
        );
        let mut guard = INITRD_DATA.lock();
        let data = InitrdData {
            frames: frame_tracker,
            size,
        };
        *guard = Some(data);
    }
}

impl Drop for InitrdData {
    fn drop(&mut self) {
        println!("Drop initrd data");
    }
}
