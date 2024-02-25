use crate::{free_frames};
use config::FRAME_SIZE;
use ksync::Mutex;

pub static INITRD_DATA: Mutex<Option<InitrdData>> = Mutex::new(None);

pub struct InitrdData {
    pub size: usize,
    pub data_ptr: usize,
}

impl Drop for InitrdData {
    fn drop(&mut self) {
        let np = (self.size + FRAME_SIZE - 1) / FRAME_SIZE;
        free_frames(self.data_ptr as *mut u8, np);
    }
}
#[cfg(feature = "initrd")]
pub(super) fn relocate_removable_data() {
    let info = platform::platform_machine_info();
    if info.initrd.is_some() {
        let start = info.initrd.as_ref().unwrap().start;
        let end = info.initrd.as_ref().unwrap().end;
        let size = end - start;
        let np = (size + FRAME_SIZE - 1) / FRAME_SIZE;
        let frame_start = crate::alloc_frames(np);
        // copy data
        unsafe {
            core::ptr::copy_nonoverlapping(start as *const u8, frame_start, size);
        }
        let mut guard = INITRD_DATA.lock();
        let data = InitrdData {
            size,
            data_ptr: frame_start as usize,
        };
        *guard = Some(data);
        println!("Relocate initrd data to {:#p}", frame_start);
    }
}
