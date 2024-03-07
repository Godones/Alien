#![no_std]
#![feature(auto_traits)]
#![feature(negative_impls)]
#![feature(error_in_core)]
#![feature(core_intrinsics)]
mod rref;
mod rvec;

extern crate alloc;

use alloc::boxed::Box;
pub use constants::AlienError;
use core::alloc::Layout;
use core::any::TypeId;
use core::error::Error;
use core::fmt::{Display, Formatter};
use log::info;
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

// impl TypeIdentifiable for [u8; 512] {
//     fn type_id() -> u64 {
//         // core::intrinsics::type_id();
//         // core::any::TypeId::of();
//         5u64
//     }
// }
//
// impl TypeIdentifiable for u8 {
//     fn type_id() -> u64 {
//         1u64
//     }
// }

impl<T: 'static> TypeIdentifiable for T {
    fn type_id() -> TypeId {
        core::any::TypeId::of::<T>()
    }
}

/// Shared heap interface
#[derive(Copy, Clone)]
pub struct SharedHeapAllocation {
    pub value_pointer: *mut u8, // *mut T
    pub domain_id_pointer: *mut u64,
    pub borrow_count_pointer: *mut u64,
    pub layout: Layout,
    pub type_id: TypeId,
}

unsafe impl Send for SharedHeapAllocation {}

pub trait SharedHeap: Send + Sync {
    unsafe fn alloc(&self, layout: Layout, type_id: TypeId) -> Option<SharedHeapAllocation>;
    unsafe fn dealloc(&self, ptr: *mut u8);
}

pub type RpcResult<T> = Result<T, RpcError>;

#[derive(Debug)]
pub enum RpcError {
    DomainCrash,
    Alien(AlienError),
}

impl From<AlienError> for RpcError {
    fn from(e: AlienError) -> Self {
        RpcError::Alien(e)
    }
}

impl Display for RpcError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            RpcError::DomainCrash => write!(f, "DomainCrash"),
            RpcError::Alien(e) => write!(f, "Alien({:?})", e),
        }
    }
}

impl Error for RpcError {}

static CRATE_DOMAIN_ID: Once<u64> = Once::new();

static HEAP: Once<Box<dyn SharedHeap>> = Once::new();

extern "C" {
    fn sbss();
    fn ebss();
}

/// 清空.bss段
fn clear_bss() {
    unsafe {
        core::slice::from_raw_parts_mut(sbss as usize as *mut u8, ebss as usize - sbss as usize)
            .fill(0);
    }
}

/// Init the shared heap
pub fn init(heap: Box<dyn SharedHeap>, domain_id: u64) {
    clear_bss();
    HEAP.call_once(|| heap);
    CRATE_DOMAIN_ID.call_once(|| domain_id);
    info!("shared heap and domain id initialized");
}

#[inline]
pub fn domain_id() -> u64 {
    *CRATE_DOMAIN_ID.get().expect("domain id not initialized")
}
