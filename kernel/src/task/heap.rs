//! 记录进程的堆空间的相关信息

/// 记录进程的堆空间的相关信息
#[derive(Debug, Clone)]
pub struct HeapInfo {
    /// 堆使用到的位置
    pub current: usize,
    /// 堆空间的起始位置
    pub start: usize,
    /// 堆空间的末尾位置
    pub end: usize,
}

impl HeapInfo {
    /// 新建一个 HeapInfo
    pub fn new(start: usize, end: usize) -> Self {
        HeapInfo {
            current: start,
            start,
            end,
        }
    }

    /// 返回堆的大小
    #[allow(unused)]
    pub fn size(&self) -> usize {
        self.end - self.start
    }

    /// 返回堆是否包括某地址
    #[allow(unused)]
    pub fn contains(&self, addr: usize) -> bool {
        addr >= self.start && addr < self.end
    }

    #[allow(unused)]
    /// 堆大小增加 size 个单位
    pub fn increase(&mut self, size: usize) {
        self.end += size;
    }

    /// 重新设置堆空间的头
    #[allow(unused)]
    pub fn set_start(&mut self, start: usize) {
        self.start = start;
    }

    #[allow(unused)]
    /// 重新设置堆空间的尾
    pub fn set_end(&mut self, end: usize) {
        self.end = end;
    }

    /// 返回堆空间是否为空
    #[allow(unused)]
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }
}
