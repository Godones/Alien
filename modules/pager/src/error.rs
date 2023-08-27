//! Error types for the buddy allocator.
use core::fmt::Display;
use core::ops::Range;

use crate::PagerResult;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum PagerError {
    OutOfMemory(usize),
    OrderTooLarge,
    PageOutOfRange,
    MemoryStartNotAligned,
    MemorySizeNotAligned,
    MemorySizeTooSmall,
    PageNotAllocated,
}

impl Display for PagerError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            PagerError::OutOfMemory(x) => write!(f, "{} Out of memory", x),
            PagerError::OrderTooLarge => write!(f, "Order too large"),
            PagerError::PageOutOfRange => write!(f, "Page out of range"),
            PagerError::MemoryStartNotAligned => write!(f, "Memory start not aligned"),
            PagerError::MemorySizeNotAligned => write!(f, "Memory size not aligned"),
            PagerError::MemorySizeTooSmall => write!(f, "Memory size too small"),
            PagerError::PageNotAllocated => write!(f, "Page not allocated"),
        }
    }
}

/// Check if the memory range is valid for the buddy allocator.
pub fn check(memory: Range<usize>) -> PagerResult<()> {
    if memory.start & 0xfff != 0 {
        return Err(PagerError::MemoryStartNotAligned);
    }
    if memory.end & 0xfff != 0 {
        return Err(PagerError::MemorySizeNotAligned);
    }
    if memory.end - memory.start < 0x1000 {
        return Err(PagerError::MemorySizeTooSmall);
    }
    Ok(())
}
