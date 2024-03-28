use core::{
    alloc::Layout,
    fmt::{Debug, Formatter},
    ops::{Deref, DerefMut},
};

use super::{RRefable, TypeIdentifiable};

#[repr(C)]
pub struct RRef<T>
where
    T: 'static + RRefable,
{
    domain_id_pointer: *mut u64,
    borrow_count_pointer: *mut u64,
    value_pointer: *mut T,
}

unsafe impl<T: RRefable> RRefable for RRef<T> {}
unsafe impl<T: RRefable> Send for RRef<T> where T: Send {}

impl<T: RRefable> RRef<T>
where
    T: TypeIdentifiable,
{
    pub unsafe fn new_with_layout(value: T, layout: Layout) -> RRef<T> {
        let type_id = T::type_id();
        let allocation = match crate::share_heap_alloc(layout, type_id) {
            Some(allocation) => allocation,
            None => panic!("Shared heap allocation failed"),
        };
        let value_pointer = allocation.value_pointer as *mut T;
        *allocation.domain_id_pointer = crate::domain_id();
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

    pub fn move_to(&self, new_domain_id: u64) {
        unsafe {
            *self.domain_id_pointer = new_domain_id;
        }
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

impl<T: RRefable> Drop for RRef<T> {
    fn drop(&mut self) {
        crate::share_heap_dealloc(self.value_pointer as *mut u8);
    }
}

impl<T: RRefable + Debug> Debug for RRef<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let value = unsafe { &*self.value_pointer };
        let domain_id = unsafe { *self.domain_id_pointer };
        let borrow_count = unsafe { *self.borrow_count_pointer };
        f.debug_struct("RRef")
            .field("value", value)
            .field("domain_id", &domain_id)
            .field("borrow_count", &borrow_count)
            .finish()
    }
}
