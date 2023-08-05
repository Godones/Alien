use core::fmt::Display;
use core::ops::Range;

use crate::BuddyResult;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum BuddyError {
    OutOfMemory(usize),
    OrderTooLarge,
    PageOutOfRange,
    MemoryStartNotAligned,
    MemorySizeNotAligned,
    MemorySizeTooSmall,
    PageNotAllocated,
}

impl Display for BuddyError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            BuddyError::OutOfMemory(x) => write!(f, "{} Out of memory", x),
            BuddyError::OrderTooLarge => write!(f, "Order too large"),
            BuddyError::PageOutOfRange => write!(f, "Page out of range"),
            BuddyError::MemoryStartNotAligned => write!(f, "Memory start not aligned"),
            BuddyError::MemorySizeNotAligned => write!(f, "Memory size not aligned"),
            BuddyError::MemorySizeTooSmall => write!(f, "Memory size too small"),
            BuddyError::PageNotAllocated => write!(f, "Page not allocated"),
        }
    }
}

pub fn check(memory: Range<usize>) -> BuddyResult<()> {
    if memory.start & 0xfff != 0 {
        return Err(BuddyError::MemoryStartNotAligned);
    }
    if memory.end & 0xfff != 0 {
        return Err(BuddyError::MemorySizeNotAligned);
    }
    if memory.end - memory.start < 0x1000 {
        return Err(BuddyError::MemorySizeTooSmall);
    }
    Ok(())
}
