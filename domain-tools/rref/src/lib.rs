#![feature(auto_traits)]
#![feature(negative_impls)]
#![no_std]
mod rref;
mod rvec;

extern crate alloc;

use alloc::boxed::Box;
use core::alloc::Layout;
use core::any::TypeId;
use core::sync::atomic::AtomicU64;
pub use rref::RRef;
pub use rvec::RRefVec;
use spin::Once;

pub unsafe auto trait RRefable {}

impl<T> !RRefable for *mut T {}
impl<T> !RRefable for *const T {}
impl<T> !RRefable for &T {}
impl<T> !RRefable for &mut T {}
impl<T> !RRefable for [T] {}

pub trait TypeIdentifiable {
    fn type_id() -> TypeId;
}

impl<T: 'static> TypeIdentifiable for T {
    fn type_id() -> TypeId {
        core::any::TypeId::of::<T>()
    }
}

#[derive(Copy, Clone)]
pub struct SharedHeapAllocation {
    pub value_pointer: *mut u8,
    pub domain_id_pointer: *mut u64,
    pub borrow_count_pointer: *mut u64,
    pub layout: Layout,
    pub type_id: TypeId,
}

unsafe impl Send for SharedHeapAllocation {}

pub trait SharedHeapAlloc: Send + Sync {
    unsafe fn alloc(&self, layout: Layout, type_id: TypeId) -> Option<SharedHeapAllocation>;
    unsafe fn dealloc(&self, ptr: *mut u8);
}

static SHARED_HEAP: Once<Box<dyn SharedHeapAlloc>> = Once::new();

static CRATE_DOMAIN_ID: AtomicU64 = AtomicU64::new(0);

pub fn init(allocator: Box<dyn SharedHeapAlloc>, domain_id: u64) {
    SHARED_HEAP.call_once(|| allocator);
    CRATE_DOMAIN_ID.store(domain_id, core::sync::atomic::Ordering::SeqCst);
}

pub fn share_heap_alloc(layout: Layout, type_id: TypeId) -> Option<SharedHeapAllocation> {
    unsafe { SHARED_HEAP.get_unchecked().alloc(layout, type_id) }
}

pub fn share_heap_dealloc(ptr: *mut u8) {
    unsafe { SHARED_HEAP.get_unchecked().dealloc(ptr) }
}

#[inline]
pub fn domain_id() -> u64 {
    CRATE_DOMAIN_ID.load(core::sync::atomic::Ordering::SeqCst)
}
