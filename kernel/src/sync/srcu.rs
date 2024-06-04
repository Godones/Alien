use core::cell::UnsafeCell;

use arch::hart_id;
use config::CPU_NUM;
use corelib::yield_now;
use ksync::Mutex;

use crate::{read_once, sync::rcu::synchronize_sched, write_once};

fn barrier() {
    //
}

fn srcu_barrier() {
    //
}
#[derive(Debug)]
pub struct SRcuArray {
    c: [UnsafeCell<u32>; 2],
}
const ARRAY_REPEAT_VALUE: UnsafeCell<u32> = UnsafeCell::new(0);

impl SRcuArray {
    pub const fn new() -> Self {
        Self {
            c: [ARRAY_REPEAT_VALUE; 2],
        }
    }
}
#[derive(Debug)]
pub struct SRcuLock {
    completed: UnsafeCell<usize>,
    per_cpu_data: [SRcuArray; CPU_NUM],
    mutex: Mutex<()>,
}

unsafe impl Sync for SRcuLock {}
unsafe impl Send for SRcuLock {}
const SRCU_ARRAY: SRcuArray = SRcuArray::new();

impl SRcuLock {
    pub const fn new() -> Self {
        Self {
            completed: UnsafeCell::new(0),
            per_cpu_data: [SRCU_ARRAY; CPU_NUM],
            mutex: Mutex::new(()),
        }
    }
    pub fn read_lock(&self) -> usize {
        let idx = read_once!(self.completed.get()) & 0x1;
        barrier();
        let hart_id = hart_id();
        let array = &self.per_cpu_data[hart_id];
        write_once!(array.c[idx].get(), read_once!(array.c[idx].get()) + 1);
        srcu_barrier();
        idx
    }

    pub fn read_unlock(&self, idx: usize) {
        srcu_barrier();
        let hart_id = hart_id();
        let array = &self.per_cpu_data[hart_id];
        // array.c[idx] -= 1;
        // assert!(read_once!(array.c[idx].get()) - 1 >= 0, "{}",read_once!(array.c[idx].get()) - 1);
        write_once!(array.c[idx].get(), read_once!(array.c[idx].get()) - 1);
    }

    fn readers_active_idx(&self, idx: usize) -> u32 {
        let mut sum = 0;
        for i in 0..CPU_NUM {
            let array = &self.per_cpu_data[i];
            sum += read_once!(array.c[idx].get());
        }
        sum
    }

    #[allow(unused)]
    fn readers_active(&self) -> u32 {
        self.readers_active_idx(0) + self.readers_active_idx(1)
    }

    pub fn synchronize(&self) {
        let idx = read_once!(self.completed.get());
        let _guard = self.mutex.lock();
        if (read_once!(self.completed.get()) - idx) > 2 {
            return;
        }
        synchronize_sched();
        let v = read_once!(self.completed.get());
        let idx = v & 0x1;
        write_once!(self.completed.get(), v + 1);
        while self.readers_active_idx(idx) != 0 {
            yield_now().unwrap(); // sleep
        }
        synchronize_sched();
    }
}
