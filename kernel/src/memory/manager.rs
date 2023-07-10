use hashbrown::HashMap;

use pager::PageAllocator;

use crate::config::{FRAME_BITS, FRAME_SIZE};
use crate::memory::FRAME_ALLOCATOR;

#[derive(Debug)]
pub struct FrameRefManager {
    record: HashMap<usize, usize>,
}

impl FrameRefManager {
    pub fn new() -> Self {
        Self {
            record: HashMap::new(),
        }
    }
    pub fn add_ref(&mut self, id: usize) -> usize {
        if let Some(count) = self.record.get_mut(&id) {
            *count += 1;
            *count
        } else {
            self.record.insert(id, 1);
            1
        }
    }
    pub fn dec_ref(&mut self, id: usize) -> Option<usize> {
        if let Some(count) = self.record.get_mut(&id) {
            *count -= 1;
            let now_count = *count;
            if *count == 0 {
                self.record.remove(&id);
                let start_addr = id << FRAME_BITS;
                unsafe {
                    core::ptr::write_bytes(start_addr as *mut u8, 0, FRAME_SIZE);
                }
                info!("free frame:{:#x}", id);
                FRAME_ALLOCATOR.lock().free(id, 0).unwrap();
            }
            return Some(now_count);
        } else {
            panic!("dec {} ref error", id);
        }
    }
    pub fn get_ref(&self, id: usize) -> usize {
        if let Some(count) = self.record.get(&id) {
            *count
        } else {
            panic!("get {} ref error", id);
        }
    }
}
