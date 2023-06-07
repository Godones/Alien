# 物理页帧分配器

此仓库包含两个页帧分配器实现：`Buddy` 与 `Bitmap`



## Interface

```rust

pub trait PageAllocator {
    /// init the allocator according to the memory range
    fn init(&mut self, memory: Range<usize>) -> BuddyResult<()>;
    /// allocate 2^order pages
    /// # Return
    /// * `OK(usize)` - the start page
    fn alloc(&mut self, order: usize) -> BuddyResult<usize>;
    /// free 2^order pages
    /// # Params
    /// * `page` - the start page
    /// * `order` - the order of pages
    fn free(&mut self, page: usize, order: usize) -> BuddyResult<()>;
}

```



```rust
pub trait PageAllocatorExt {
    /// allocate pages
    /// # Params
    /// * `pages` - the number of pages, it may not be 2^order
    fn alloc_pages(&mut self, pages: usize) -> BuddyResult<usize>;
    /// free pages
    /// # Params
    /// * `page` - the start page
    /// * `pages` - the number of pages, it may not be 2^order
    fn free_pages(&mut self, page: usize, pages: usize) -> BuddyResult<()>;
}

```



## Buddy

伙伴系统是Linux中经典的页帧分配器，这里对其进行了简化，去掉了高速缓存，以及迁移类型等复杂机制。

![11964835-6ec33b050ad5d51e](assert/11964835-6ec33b050ad5d51e.webp)

这里的实现使用了`doubly-linked-list`库，这是一个类似Linux中的双链表结构，每一物理块首页都后包含一个链表项：

```
pub struct ListHead {
    pub prev: *mut ListHead,
    pub next: *mut ListHead,
}
```

通过这个数据结构，可以将所有的物理块串联起来。

物理页的分配按照2的幂次进行，在当前幂次不足时向下一个幂次进行请求。

<img src="assert/11964835-a3f9d2ff8945fef3.webp" alt="11964835-a3f9d2ff8945fef3" style="zoom:50%;" />

物理页的释放会触发合并操作，以便后续更大的页请求可以得到满足。

<img src="assert/11964835-dfdcb744fea83137.webp" alt="11964835-dfdcb744fea83137" style="zoom:50%;" />

## Bitmap

位图分配器使用一个大数组的bit位来管理所有页面。其分配和释放算法都很简单。









## Feature

- [ ] 为buddy加入高速缓存
- [ ] bitmap更快的查找算法

