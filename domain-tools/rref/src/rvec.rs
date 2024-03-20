use super::{RRef, RRefable, TypeIdentifiable};
use core::alloc::Layout;
use core::fmt::{Debug, Formatter};
use core::mem::MaybeUninit;
use core::ops::{Deref, DerefMut};

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

impl<T> Debug for RRefVec<T>
where
    T: 'static + RRefable + Copy + TypeIdentifiable + Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("RRefVec")
            .field("data", &self.data)
            .field("size", &self.size)
            .finish()
    }
}
