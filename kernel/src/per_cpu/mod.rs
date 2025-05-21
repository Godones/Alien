use alloc::{boxed::Box, vec, vec::Vec};
use core::fmt::{Debug, Formatter};

use arch::hart_id;
use bpf_basic::map::{PerCpuVariants, PerCpuVariantsOps};
use config::CPU_NUM;

#[derive(Debug)]
pub struct PerCpuImpl;
impl PerCpuVariantsOps for PerCpuImpl {
    fn create<T: Clone + Sync + Send + 'static>(value: T) -> Option<Box<dyn PerCpuVariants<T>>> {
        let data = PerCpuVariantsImpl::new_with_value(value);
        Some(Box::new(data))
    }

    fn num_cpus() -> u32 {
        CPU_NUM as u32
    }
}

pub struct PerCpuVariantsImpl<T> {
    data: Vec<T>,
}

impl<T: Send + Sync + Clone> PerCpuVariantsImpl<T> {
    pub fn new() -> Self {
        Self {
            data: Vec::with_capacity(CPU_NUM),
        }
    }
    pub fn new_with_value(value: T) -> Self {
        Self {
            data: vec![value; CPU_NUM as usize],
        }
    }
}

impl<T> Debug for PerCpuVariantsImpl<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PerCpuVariantsImpl").finish()
    }
}

impl<T: Send + Sync + Clone> PerCpuVariants<T> for PerCpuVariantsImpl<T> {
    fn get(&self) -> &T {
        &self.data[hart_id()]
    }

    fn get_mut(&self) -> &mut T {
        unsafe { &mut (self as *const Self as *mut Self).as_mut().unwrap().data[hart_id()] }
    }

    unsafe fn force_get(&self, cpu: u32) -> &T {
        &self.data[cpu as usize]
    }

    unsafe fn force_get_mut(&self, cpu: u32) -> &mut T {
        unsafe { &mut (self as *const Self as *mut Self).as_mut().unwrap().data[cpu as usize] }
    }
}
