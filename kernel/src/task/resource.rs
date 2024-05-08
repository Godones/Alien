use config::MAX_THREAD_NUM;
use ksync::Mutex;
use small_index::IndexAllocator;
use spin::Lazy;

/// 这里把MinimalManager复用为tid分配器，通常，MinimalManager会将数据插入到最小可用位置并返回位置，
/// 但tid的分配并不需要实际存储信息，因此可以插入任意的数据，这里为了节省空间，将数据定义为u8
pub static TID_MANAGER: Lazy<Mutex<IndexAllocator<MAX_THREAD_NUM>>> =
    Lazy::new(|| Mutex::new(IndexAllocator::new()));
/// 用于存储线程的tid
#[derive(Debug)]
pub struct TidHandle(pub usize);

impl TidHandle {
    /// 获取一个新的线程 tid (来自于 `TID_MANAGER` 分配)
    pub fn new() -> Option<Self> {
        let tid = TID_MANAGER.lock().allocate().ok();
        tid.map(|tid| TidHandle(tid))
    }
    #[allow(unused)]
    pub fn raw(&self) -> usize {
        self.0
    }
}

impl Drop for TidHandle {
    fn drop(&mut self) {
        TID_MANAGER.lock().deallocate(self.0).unwrap();
    }
}

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
    pub fn new(start: usize, end: usize) -> Self {
        HeapInfo {
            current: start,
            start,
            end,
        }
    }

    pub fn contains(&self, addr: usize) -> bool {
        addr >= self.start && addr < self.end
    }
}
