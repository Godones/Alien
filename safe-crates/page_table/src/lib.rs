#![no_std]
#![feature(doc_auto_cfg)]
#![feature(doc_cfg)]
#![feature(const_trait_impl)]
#![forbid(unsafe_code)]
mod page_table_entry;

#[macro_use]
extern crate log;
extern crate alloc;

mod bits64;
mod riscv;

use alloc::boxed::Box;

use memory_addr::{PhysAddr, VirtAddr};
#[doc(no_inline)]
pub use page_table_entry::{riscv::Rv64PTE, GenericPTE, MappingFlags};
pub use riscv::*;

pub use self::bits64::{PageTable64, ENTRY_COUNT};

/// The error type for page table operation failures.
#[derive(Debug)]
pub enum PagingError {
    /// Cannot allocate memory.
    NoMemory,
    /// The address is not aligned to the page size.
    NotAligned,
    /// The mapping is not present.
    NotMapped,
    /// The mapping is already present.
    AlreadyMapped,
    /// The page table entry represents a huge page, but the target physical
    /// frame is 4K in size.
    MappedToHugePage,
}

/// The specialized `Result` type for page table operations.
pub type PagingResult<T = ()> = Result<T, PagingError>;

/// The **architecture-dependent** metadata that must be provided for
/// [`PageTable64`].
#[const_trait]
pub trait PagingMetaData: Sync + Send + Sized {
    /// The number of levels of the hardware page table.
    const LEVELS: usize;
    /// The maximum number of bits of physical address.
    const PA_MAX_BITS: usize;
    /// The maximum number of bits of virtual address.
    const VA_MAX_BITS: usize;

    /// The maximum physical address.
    const PA_MAX_ADDR: usize = (1 << Self::PA_MAX_BITS) - 1;

    /// Whether a given physical address is valid.
    #[inline]
    fn paddr_is_valid(paddr: usize) -> bool {
        paddr <= Self::PA_MAX_ADDR // default
    }

    /// Whether a given virtual address is valid.
    #[inline]
    fn vaddr_is_valid(vaddr: usize) -> bool {
        // default: top bits sign extended
        let top_mask = usize::MAX << (Self::VA_MAX_BITS - 1);
        (vaddr & top_mask) == 0 || (vaddr & top_mask) == top_mask
    }
}

pub trait NotLeafPage<PTE: GenericPTE>: Send + Sync {
    /// Returns the physical address
    fn phys_addr(&self) -> PhysAddr;
    /// Returns a virtual address that maps to the given physical address.
    ///
    /// Used to access the physical memory directly in page table implementation.
    fn virt_addr(&self) -> VirtAddr;
    /// Zero the page.
    fn zero(&self);
    fn as_pte_slice<'a>(&self) -> &'a [PTE];
    fn as_pte_mut_slice<'a>(&self) -> &'a mut [PTE];
}

/// The low-level **OS-dependent** helpers that must be provided for
/// [`PageTable64`].
pub trait PagingIf<PTE: GenericPTE>: Sized {
    /// Request to allocate a 4K-sized physical frame.
    fn alloc_frame() -> Option<Box<dyn NotLeafPage<PTE>>>;
}

/// The page sizes supported by the hardware page table.
#[repr(usize)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum PageSize {
    /// Size of 4 kilobytes (2<sup>12</sup> bytes).
    Size4K = 0x1000,
    /// Size of 2 megabytes (2<sup>21</sup> bytes).
    Size2M = 0x20_0000,
    /// Size of 1 gigabytes (2<sup>30</sup> bytes).
    Size1G = 0x4000_0000,
}

impl PageSize {
    /// Whether this page size is considered huge (larger than 4K).
    pub const fn is_huge(self) -> bool {
        matches!(self, Self::Size1G | Self::Size2M)
    }
}

impl From<PageSize> for usize {
    #[inline]
    fn from(size: PageSize) -> usize {
        size as usize
    }
}
