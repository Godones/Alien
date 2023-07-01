#[derive(Debug, Clone)]
pub struct HeapInfo {
    pub current: usize,
    pub start: usize,
    pub end: usize,
}

impl HeapInfo {
    pub fn new(start: usize, end: usize) -> Self {
        HeapInfo {
            current: start,
            start,
            end,
        }
    }

    #[allow(unused)]
    pub fn size(&self) -> usize {
        self.end - self.start
    }

    #[allow(unused)]
    pub fn contains(&self, addr: usize) -> bool {
        addr >= self.start && addr < self.end
    }

    pub fn increase(&mut self, size: usize) {
        self.end += size;
    }

    #[allow(unused)]
    pub fn set_start(&mut self, start: usize) {
        self.start = start;
    }

    pub fn set_end(&mut self, end: usize) {
        self.end = end;
    }

    #[allow(unused)]
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }
}
