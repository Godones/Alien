#[derive(Debug, Clone)]
pub struct IoVec {
    pub base: *mut u8,
    pub len: usize,
}
