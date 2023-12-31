#![no_std]
#![feature(auto_traits)]
#![feature(negative_impls)]
#![feature(error_in_core)]
extern crate alloc;

use alloc::boxed::Box;
use core::alloc::Layout;
use core::error::Error;
use core::fmt::{Display, Formatter};
use core::ops::{Deref, DerefMut};
use spin::Once;

pub unsafe auto trait RRefable {}
impl<T> !RRefable for *mut T {}
impl<T> !RRefable for *const T {}
impl<T> !RRefable for &T {}
impl<T> !RRefable for &mut T {}
impl<T> !RRefable for [T] {}

pub trait TypeIdentifiable {
    fn type_id() -> u64;
}

#[repr(C)]
pub struct RRef<T>
where
    T: 'static + RRefable,
{
    domain_id_pointer: *mut u64,
    pub(crate) borrow_count_pointer: *mut u64,
    pub(crate) value_pointer: *mut T,
}

unsafe impl<T: RRefable> RRefable for RRef<T> {}
unsafe impl<T: RRefable> Send for RRef<T> where T: Send {}

impl<T: RRefable> RRef<T>
where
    T: TypeIdentifiable,
{
    unsafe fn new_with_layout(value: T, layout: Layout) -> RRef<T> {
        let type_id = T::type_id();
        let allocation = match unsafe {
            HEAP.get()
                .expect("Shared heap not initialized")
                .alloc(layout, type_id)
        } {
            Some(allocation) => allocation,
            None => panic!("Shared heap allocation failed"),
        };
        let value_pointer = allocation.value_pointer as *mut T;
        *allocation.domain_id_pointer = libsyscall::domain_id();
        *allocation.borrow_count_pointer = 0;
        core::ptr::write(value_pointer, value);
        RRef {
            domain_id_pointer: allocation.domain_id_pointer,
            borrow_count_pointer: allocation.borrow_count_pointer,
            value_pointer,
        }
    }
    pub fn new(value: T) -> RRef<T> {
        let layout = Layout::new::<T>();
        unsafe { Self::new_with_layout(value, layout) }
    }
    pub fn new_aligned(value: T, align: usize) -> RRef<T> {
        let size = core::mem::size_of::<T>();
        let layout = unsafe { Layout::from_size_align_unchecked(size, align) };
        unsafe { Self::new_with_layout(value, layout) }
    }
}

impl<T: RRefable> Deref for RRef<T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.value_pointer }
    }
}
impl<T: RRefable> DerefMut for RRef<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.value_pointer }
    }
}

impl TypeIdentifiable for [u8; 512] {
    fn type_id() -> u64 {
        5u64
    }
}

/// Shared heap interface
#[derive(Copy, Clone)]
pub struct SharedHeapAllocation {
    pub value_pointer: *mut u8, // *mut T
    pub domain_id_pointer: *mut u64,
    pub borrow_count_pointer: *mut u64,
    pub layout: Layout,
    pub type_id: u64,
}

unsafe impl Send for SharedHeapAllocation {}

pub trait SharedHeap: Send + Sync {
    unsafe fn alloc(&self, layout: Layout, type_id: u64) -> Option<SharedHeapAllocation>;
    unsafe fn dealloc(&self, ptr: *mut u8);
}

pub type RpcResult<T> = Result<T, RpcError>;

#[derive(Debug)]
pub enum RpcError {
    DomainCrash,
}
impl Display for RpcError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            RpcError::DomainCrash => write!(f, "DomainCrash"),
        }
    }
}

impl Error for RpcError {}

static HEAP: Once<Box<dyn SharedHeap>> = Once::new();

pub fn init(heap: Box<dyn SharedHeap>) {
    HEAP.call_once(|| heap);
}
