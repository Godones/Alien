use alloc::collections::BTreeMap;
use pager::PageAllocator;

use crate::memory::FRAME_ALLOCATOR;

#[derive(Debug)]
pub struct FrameRefManager {
    record: BTreeMap<usize, usize>,
}

impl FrameRefManager {
    pub fn new() -> Self {
        Self {
            record: BTreeMap::new(),
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
