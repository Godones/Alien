use core::ops::Range;
use page_table::pte::MappingFlags;

#[derive(Debug)]
pub struct VmArea {
    range: Range<usize>,
    permission: MappingFlags,
}

impl VmArea {
    pub fn new(range: Range<usize>, permission: MappingFlags) -> Self {
        Self { range, permission }
    }
    pub fn range(&self) -> Range<usize> {
        self.range.clone()
    }
    pub fn permission(&self) -> MappingFlags {
        self.permission
    }
}
