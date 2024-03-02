use crate::{RRef, RRefable, TypeIdentifiable};
use core::alloc::Layout;
use core::mem::MaybeUninit;
use core::ops::{Deref, DerefMut};

#[doc = " `RRef`ed runtime constant size array."]
#[doc = " This allow us to pass array across domains without having"]
#[doc = " its size being limited at complie time like in RRefArray."]
#[doc = ""]
#[doc = " Currently, it only support Copy types since we only need"]
#[doc = " it for passing byte arrays around. We will later merge it"]
#[doc = " with RRefArray when we have time."]
pub struct RRefVec<T>
where
    T: 'static + RRefable + Copy + TypeIdentifiable,
{
    data: RRef<T>,
    size: usize,
}
unsafe impl<T> RRefable for RRefVec<T> where T: 'static + RRefable + Copy + TypeIdentifiable {}
unsafe impl<T> Send for RRefVec<T> where T: 'static + RRefable + Copy + TypeIdentifiable {}
impl<T> RRefVec<T>
where
    T: 'static + RRefable + Copy + TypeIdentifiable,
{
    pub fn new(initial_value: T, size: usize) -> Self {
        let layout = Layout::array::<T>(size).unwrap();
        let data = unsafe { RRef::new_with_layout(initial_value, layout) };
        let mut vec = Self { data, size };
        vec.as_mut_slice().fill(initial_value);
        vec
    }
    pub fn from_slice(slice: &[T]) -> Self {
        let size = slice.len();
        let layout = Layout::array::<T>(size).unwrap();
        let data = unsafe { RRef::new_with_layout(MaybeUninit::uninit().assume_init(), layout) };
        let mut vec = Self { data, size };
        for (dest, src) in vec.as_mut_slice().iter_mut().zip(slice) {
            *dest = *src;
        }
        vec
    }
    pub fn as_slice(&self) -> &[T] {
        unsafe { core::slice::from_raw_parts(&*self.data, self.size) }
    }
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { core::slice::from_raw_parts_mut(&mut *self.data, self.size) }
    }
    pub fn size(&self) -> usize {
        self.size
    }
    pub fn len(&self) -> usize {
        self.size
    }
    pub fn move_to(&self, new_domain_id: u64) {
        self.data.move_to(new_domain_id);
    }
}

impl<T> Deref for RRefVec<T>
where
    T: 'static + RRefable + Copy + TypeIdentifiable,
{
    type Target = T;
    fn deref(&self) -> &T {
        &self.data
    }
}
impl<T> DerefMut for RRefVec<T>
where
    T: 'static + RRefable + Copy + TypeIdentifiable,
{
    fn deref_mut(&mut self) -> &mut T {
        &mut self.data
    }
}
