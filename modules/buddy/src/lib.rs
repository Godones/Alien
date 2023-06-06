#![feature(generic_const_exprs)]
#![cfg_attr(not(test), no_std)]
#![allow(incomplete_features)]
//! Buddy memory allocator


extern crate alloc;

#[cfg(feature = "buddy")]
pub use crate::buddy::Zone;

mod buddy;
mod error;
mod bitmap;

type BuddyResult<T> = Result<T, error::BuddyError>;


pub trait PageAllocator {
    /// allocate 2^order pages
    /// # Return
    /// * `OK(usize)` - the start page
    fn alloc(&mut self, order: usize) -> BuddyResult<usize>;
    /// free 2^order pages
    /// # Params
    /// * `page` - the start page
    /// * `order` - the order of pages
    fn free(&mut self, page: usize, order: usize) -> BuddyResult<()>;
}