#![no_std]
#![forbid(unsafe_code)]
mod area;
mod space;

extern crate alloc;

use core::fmt::Debug;

pub use area::{VmArea, VmAreaEqual, VmAreaType};
use memory_addr::{PhysAddr, VirtAddr};
use page_table::PagingResult;
use pod::Pod;
pub use space::VmSpace;

pub trait PhysPage: Debug + Send + Sync {
    fn phys_addr(&self) -> PhysAddr;
    fn as_bytes(&self) -> &[u8];
    fn as_mut_bytes(&mut self) -> &mut [u8];
}

pub trait VmIo {
    /// Read a specified number of bytes at a specified offset into a given buffer.
    ///
    /// # No short reads
    ///
    /// On success, the output `buf` must be filled with the requested data
    /// completely. If, for any reason, the requested data is only partially
    /// available, then the method shall return an error.
    fn read_bytes(&self, offset: VirtAddr, buf: &mut [u8]) -> PagingResult<()>;

    /// Read a value of a specified type at a specified offset.
    fn read_val<T: Pod>(&self, offset: VirtAddr) -> PagingResult<T> {
        let mut val = T::new_uninit();
        self.read_bytes(offset, val.as_bytes_mut())?;
        Ok(val)
    }

    /// Read a slice of a specified type at a specified offset.
    ///
    /// # No short reads
    ///
    /// Similar to `read_bytes`.
    fn read_slice<T: Pod>(&self, offset: VirtAddr, slice: &mut [T]) -> PagingResult<()> {
        let size = core::mem::size_of::<T>();
        let mut offset = offset;
        for i in 0..slice.len() {
            let tmp = self.read_val(offset)?;
            slice[i] = tmp;
            offset += size;
        }
        Ok(())
    }

    /// Write a specified number of bytes from a given buffer at a specified offset.
    ///
    /// # No short writes
    ///
    /// On success, the input `buf` must be written to the VM object entirely.
    /// If, for any reason, the input data can only be written partially,
    /// then the method shall return an error.
    fn write_bytes(&mut self, offset: VirtAddr, buf: &[u8]) -> PagingResult<()>;

    /// Write a value of a specified type at a specified offset.
    fn write_val<T: Pod>(&mut self, offset: VirtAddr, new_val: &T) -> PagingResult<()> {
        self.write_bytes(offset, new_val.as_bytes())?;
        Ok(())
    }

    /// Write a slice of a specified type at a specified offset.
    ///
    /// # No short write
    ///
    /// Similar to `write_bytes`.
    fn write_slice<T: Pod>(&mut self, offset: VirtAddr, slice: &[T]) -> PagingResult<()> {
        let size = core::mem::size_of::<T>();
        let mut offset = offset;
        for i in 0..slice.len() {
            self.write_val(offset, &slice[i])?;
            offset += size;
        }
        Ok(())
    }
}
