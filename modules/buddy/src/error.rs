use core::fmt::Display;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum BuddyError {
    OutOfMemory,
    OrderTooLarge,
    PageOutOfRange,
    MemoryStartNotAligned,
    MemorySizeNotAligned,
    MemorySizeTooSmall,
}

impl Display for BuddyError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            BuddyError::OutOfMemory => write!(f, "Out of memory"),
            BuddyError::OrderTooLarge => write!(f, "Order too large"),
            BuddyError::PageOutOfRange => write!(f, "Page out of range"),
            BuddyError::MemoryStartNotAligned => write!(f, "Memory start not aligned"),
            BuddyError::MemorySizeNotAligned => write!(f, "Memory size not aligned"),
            BuddyError::MemorySizeTooSmall => write!(f, "Memory size too small")
        }
    }
}

