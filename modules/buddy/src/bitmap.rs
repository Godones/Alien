use crate::{BuddyResult, PageAllocator};

pub struct BitMap<const SIZE: usize>
    where
        [(); SIZE / 8 + 1]:,
{
    map: [u8; SIZE / 8 + 1],
}


impl<const SIZE: usize> PageAllocator for BitMap<SIZE>
    where
        [(); SIZE / 8 + 1]:,
{
    fn alloc(&mut self, order: usize) -> BuddyResult<usize> {
        todo!()
    }
    fn free(&mut self, page: usize, order: usize) -> BuddyResult<()> {
        todo!()
    }
}


impl<const SIZE: usize> BitMap<SIZE>
    where
        [(); SIZE / 8 + 1]:,
{}