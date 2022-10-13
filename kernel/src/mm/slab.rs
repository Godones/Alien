use crate::mm::alloc_frames;
use core::cmp::{max, min};
use core::fmt::{Debug, Formatter};
use core::mem::forget;
use core::ops::{Add};
use double_link_list::*;



const CACHE_NAME_MAX: usize = 16;
const FRAME_SIZE: usize = 0x1000; //4KB


#[allow(unused)]
pub fn init_slab_allocator() {
    // todo!(slab分配器实现)
}

// 管理所有的list_head链表
pub static mut SLAB_CACHES: ListHead = ListHead::new();
static mut MEM_CACHE_BOOT: MemCache = MemCache::new();

/// Cache define\
/// per_objects:每个slab的对象数量\
/// per_frames: 每个slab的页帧数量 2^per_frames\
/// object_size: 对象大小\
/// mem_cache_node: Slab管理节点\
/// cache_name: Cache名称\
pub struct MemCache {
    list: ListHead,
    per_objects: u32,
    per_frames: u32,
    align: u32,
    object_size: u32,
    mem_cache_node: CacheNode,
    cache_name: [u8; CACHE_NAME_MAX],
} //32+48+16 = 96

impl Debug for MemCache {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let str = core::str::from_utf8(&self.cache_name).unwrap();
        f.write_fmt(format_args!(
            "mem_cache{{\n\
        \tlist:{:?}\n\
        \tper_objects:{:?}\n\
        \tper_frames:{:?}\n\
        \talign:{:?}\n\
        \tobject_size:{:?}\n\
        \tmem_cache_node:{:?}\n\
        \tcache_name:{:?}\
        }}",
            self.list,
            self.per_objects,
            self.per_frames,
            self.align,
            self.object_size,
            self.mem_cache_node,
            str
        ))
    }
}

/// Cache Node define\
/// slab_partial: 部分分配链表\
/// slab_free: 空Slab/未分配\
/// slab_full: 完全分配\
///
struct CacheNode {
    slab_partial: ListHead,
    slab_free: ListHead,
    slab_full: ListHead,
}

impl CacheNode {
    pub const fn new() -> Self {
        CacheNode {
            slab_partial: ListHead::new(),
            slab_free: ListHead::new(),
            slab_full: ListHead::new(),
        }
    }
}

impl Debug for CacheNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!(
            "CacheNode {{ \n\
            \tslab_partial: {:?},\n\
            \tslab_free: {:?}, \n\
            \tslab_full: {:?} \
         }}",
            self.slab_partial, self.slab_free, self.slab_full
        ))
    }
}

/// Slab define\
/// cache: 指向所属的Cache\
/// used_object：已分配的对象数量\
/// next_free: 下一个空闲的对象\
/// first_object: 第一个对象的地址\
/// free_list: 数组索引用来记录空闲的对象\
///
struct Slab {
    list: ListHead,
    cache: *mut MemCache,
    used_object: u32,
    next_free: u32,
    fist_object: usize,
    free_list: *mut u32,
}

impl Debug for Slab {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!(
            "Slab{{\n\
            \tlist:{:?},\n\
            \tcache:{:?},\n\
            \tused_object:{},\n\
            \tnext_free:{},\n\
            \tfist_object:{:#x},\n\
            \tfree_list:{:?}\
            }}",
            self.list,
            self.cache,
            self.used_object,
            self.next_free,
            self.fist_object,
            self.free_list
        ))
    }
}

impl Slab {
    pub fn new(cache: &MemCache) {
        // 创建一个slab
        // 从cache获取需要申请的页面和对象大小
        // 申请页面
        // 初始化slab
        // 将slab添加到cache的slab_partial链表中
        let per_frames = cache.per_frames;
        let start_addr = alloc_frames_for_cache(per_frames) as usize;
        // todo!(需要处理align过大的情况)
        // 对象从后往前分配
        debug!("alloc frame start_addr:{:x}", start_addr);
        // todo!(应该在cache中记录第一个对象的地址？)
        let slab_manager_align_addr =
            MemCache::slab_align_addr(cache.per_objects as usize, cache.align as usize);
        let fist_object_addr = start_addr.add(slab_manager_align_addr);

        let free_list_addr = start_addr.add(core::mem::size_of::<Slab>());
        info!(
            "slab start_addr:{:x}, fist_object_addr:{:x}, free_list_addr:{:x}",
            start_addr, fist_object_addr, free_list_addr
        );
        let slab = Slab {
            list: ListHead::new(),
            cache: cache as *const MemCache as *mut MemCache,
            used_object: 0,
            next_free: 0,
            fist_object: fist_object_addr as usize,
            free_list: free_list_addr as *mut u32,
        };
        // 写入slab信息到开始位置
        unsafe {
            core::ptr::write_volatile(start_addr as *mut Slab, slab);
            // 初始化free_list
            for i in 0..cache.per_objects {
                core::ptr::write_volatile(
                    free_list_addr.add(i as usize * core::mem::size_of::<u32>()) as *mut u32,
                    i,
                );
            }
        }
        let slab = unsafe { &mut *(start_addr as *mut Slab) };
        list_head_init!(slab.list);
        trace!("slab:{:?}", slab);
        // 加入到cache的slab_free链表中
        list_add_tail!(
            to_list_head_ptr!(slab.list),
            to_list_head_ptr!(cache.mem_cache_node.slab_free)
        );
    }

    pub fn alloc(&mut self) -> *mut u8 {
        let cache = unsafe { &mut *self.cache };
        let per_objects = cache.per_objects;
        if self.next_free < per_objects {
            let pos = unsafe { self.free_list.add(self.next_free as usize).read_volatile() };
            let addr = self
                .fist_object
                .add(pos as usize * cache.object_size as usize);
            self.next_free += 1;
            self.used_object += 1;
            return addr as *mut u8;
        }
        core::ptr::null_mut()
    }
    pub fn dealloc(&mut self, addr: *mut u8) {
        let cache = unsafe { &mut *self.cache };
        let pos = (addr as usize - self.fist_object) / cache.object_size as usize;
        self.next_free -= 1;
        unsafe {
            self.free_list
                .add(self.next_free as usize)
                .write_volatile(pos as u32);
        }
        self.used_object -= 1;
    }
    fn start(&self) -> usize {
        // 返回slab的起始地址
        self as *const Slab as usize
    }

    #[inline]
    pub fn move_to(&mut self, to: *mut ListHead) {
        list_del!(to_list_head_ptr!(self.list));
        list_add_tail!(to_list_head_ptr!(self.list), to);
    }
    pub fn is_in_slab(&self, addr: *mut u8) -> bool {
        //检查此地址是否位于slab中
        let addr = addr as usize;
        let cache = unsafe { &mut *self.cache };
        let start_addr = self.start() as *const Slab as usize;
        let end_addr = start_addr.add(cache.per_frames as usize * FRAME_SIZE);
        (start_addr <= addr) && (addr < end_addr)
    }
}

impl MemCache {
    pub const fn new() -> Self {
        Self {
            list: ListHead::new(),
            per_objects: 0,
            per_frames: 0,
            align: 0,
            object_size: 0,
            mem_cache_node: CacheNode::new(),
            cache_name: [0; CACHE_NAME_MAX],
        }
    }
    /// 打印信息
    fn print_info(&self) {
        let index = self
            .cache_name
            .iter()
            .position(|&x| x == 0)
            .unwrap_or(CACHE_NAME_MAX);
        let str = core::str::from_utf8(&self.cache_name[0..index]).unwrap();
        // 计算总的对象和已使用的对象
        let mut total = 0;
        let mut used = 0;
        let per_objects = self.per_objects as usize;
        total += (self.mem_cache_node.slab_free.len()
            + self.mem_cache_node.slab_full.len()
            + self.mem_cache_node.slab_partial.len())
            * per_objects;
        self.mem_cache_node.slab_partial.iter().for_each(|slab| {
            let slab = unsafe { &*container_of!(slab as usize, Slab, list) };
            used += slab.used_object as usize;
        });
        used += self.mem_cache_node.slab_full.len() * per_objects;
        println!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}",
            str, self.object_size, self.per_frames, self.per_objects, self.align, used, total
        );
    }

    #[inline]
    fn cache_node_init(&mut self) {
        list_head_init!(self.mem_cache_node.slab_partial);
        list_head_init!(self.mem_cache_node.slab_free);
        list_head_init!(self.mem_cache_node.slab_full);
    }
    #[inline]
    fn cache_name_init(&mut self, name: &[u8]) {
        let mut cache_name = [0u8; CACHE_NAME_MAX];
        for i in 0..(min(name.len(), CACHE_NAME_MAX)) {
            cache_name[i] = name[i];
        }
        self.cache_name = cache_name;
    }

    fn slab_align_addr(object_num: usize, align: usize) -> usize {
        align_to!(
            object_num * core::mem::size_of::<u32>() + core::mem::size_of::<Slab>(),
            align
        )
    }

    /// 计算每个slab中对象的数量
    fn init_cache_object_num(&mut self) {
        let slab_size = FRAME_SIZE * self.per_frames as usize;
        let mut object_num = (slab_size - core::mem::size_of::<Slab>())
            / (self.object_size as usize + core::mem::size_of::<u32>());
        // 计算cache line对齐后slab管理对象的大小
        while MemCache::slab_align_addr(object_num, self.align as usize) > slab_size {
            object_num -= 1;
        }
        trace!(
            "slab_manager_size:{}",
            MemCache::slab_align_addr(object_num, self.align as usize)
        );
        self.per_objects = object_num as u32;
    }

    pub fn init(&mut self, name: &[u8], object_size: u32, per_frames: u32, align: u32) {
        self.cache_node_init();
        self.cache_name_init(name);
        self.object_size = object_size;
        // 在每个slab中，每次可能会分配1到多个页帧
        // 在每次页帧的最后一个页帧中，会存在一个slab结构体用来记录
        // 该slab的信息
        self.per_frames = 1 << per_frames;
        let align = if align.is_power_of_two() && align != 0 {
            max(align, 8)
        } else {
            core::mem::size_of::<usize>() as u32
        };
        self.align = align;
        // 分配的物理页帧起始位置由
        // slab结构体 + free_list数组构成
        // 第一个对象的地址需要对齐到align
        self.init_cache_object_num();
    }

    pub fn alloc(&mut self) -> *mut u8 {
        // 先检查partial链表
        let mut slab_list = to_list_head_ptr!(self.mem_cache_node.slab_partial);
        let slab = if !is_list_empty!(slab_list) {
            // 非空则从slab中分配
            slab_list = self.mem_cache_node.slab_partial.next; //第一个可用slab
            let slab = unsafe { &mut (*container_of!(slab_list as usize, Slab, list)) };
            slab
        } else if is_list_empty!(to_list_head_ptr!(self.mem_cache_node.slab_free)) {
            // 如果partial链表为空，则检查free链表
            // 如果free链表也为空，则需要分配新的slab
            // 需要直接从vmalloc中分配页面过来
            debug!("alloc new slab");
            Slab::new(self); // 创建新的slab,并加入到cache的free链表中
            assert!(!is_list_empty!(to_list_head_ptr!(
                self.mem_cache_node.slab_free
            )));
            slab_list = self.mem_cache_node.slab_free.next; //第一个可用slab
            let slab = unsafe { &mut (*container_of!(slab_list as usize, Slab, list)) };
            slab.move_to(to_list_head_ptr!(self.mem_cache_node.slab_partial));
            slab
        } else {
            // 如果free链表不为空，则将free链表中的slab移动到partial链表中
            slab_list = self.mem_cache_node.slab_free.next;
            let slab = unsafe { &mut (*container_of!(slab_list as usize, Slab, list)) };
            // 将slab移动到partial部分
            slab.move_to(to_list_head_ptr!(self.mem_cache_node.slab_partial));
            slab
        };
        // 从slab中分配
        let addr = slab.alloc();
        if slab.used_object == self.per_objects {
            // 如果slab中的对象已经全部分配完毕，则将slab移动到full链表中
            slab.move_to(to_list_head_ptr!(self.mem_cache_node.slab_full));
        }
        addr
    }
    pub fn dealloc(&mut self, addr: *mut u8) {
        // 查找此对象所在的slab
        // todo!(如何查找？)
        // 这个地址可能位于partial / full
        self.mem_cache_node.slab_partial.iter().for_each(|slab_list| {
            let slab = unsafe { &mut (*container_of!(slab_list as usize, Slab, list)) };
            if slab.is_in_slab(addr) {
                slab.dealloc(addr);
                if slab.used_object == 0 {
                    // 如果slab中的对象已经全部释放，则将slab移动到free链表中
                    slab.move_to(to_list_head_ptr!(self.mem_cache_node.slab_free));
                }
            }
        });
        self.mem_cache_node.slab_full.iter().for_each(|slab_list| {
            let slab = unsafe { &mut (*container_of!(slab_list as usize, Slab, list)) };
            if slab.is_in_slab(addr) {
                slab.dealloc(addr);
                if slab.used_object == 0 {
                    slab.move_to(to_list_head_ptr!(self.mem_cache_node.slab_free));
                } else {
                    slab.move_to(to_list_head_ptr!(self.mem_cache_node.slab_partial));
                }
                return;
            }
        });
    }
}

fn alloc_frames_for_cache(pages: u32) -> *mut u8 {
    // 直接从页帧分配器中分配连续的pages个页面
    trace!("alloc for cache:{}", pages);
    let f = alloc_frames(pages as usize).unwrap();
    let start = f.start() as *mut u8;
    forget(f);
    start
}

/// 外部的页帧管理器可以通过这个接口来回收slab中的页帧
/// f:此函数是外部的回收页帧调用的函数
pub fn reclaim_frame_from_cache(fn_dealloc: fn(start: usize, count: usize)) {
    // 需要SLAB_CACHES链表中找到存在空闲SLAB的cache
    // 然后从里面回收相关的页帧
    let cache_list = unsafe { &SLAB_CACHES };
    cache_list.iter().for_each(|cache_list| {
        let cache = unsafe { &mut (*container_of!(cache_list as usize, MemCache, list)) };
        let slab_list = &cache.mem_cache_node.slab_free;
        slab_list.iter().for_each(|slab_list| {
            let slab = unsafe { &mut (*container_of!(slab_list as usize, Slab, list)) };
            // 回收slab中的页帧
            fn_dealloc(slab.start(), cache.per_frames as usize);
            // 将slab从free链表中移除
            list_del!(slab_list);
        });
    });
}

pub fn create_cache(name: &[u8], object_size: u32, per_frames: u32, align: u32) -> *mut MemCache {
    // 创建一个自定义cache
    // 从第一个初始化的cache中分配一个cached对象
    let cache = unsafe { &mut MEM_CACHE_BOOT };
    let cache_object_addr = cache.alloc() as *mut MemCache;
    let cache_object = unsafe { &mut (*cache_object_addr) };
    // 初始化cache
    cache_object.init(name, object_size, per_frames, align);
    // 将cache加入到SLAB_CACHES链表中
    list_add_tail!(
        to_list_head_ptr!(cache_object.list),
        to_list_head_ptr!(SLAB_CACHES)
    );
    cache_object_addr
}

/// 打印系统内的所有cache 信息
/// 格式:
/// cache_name object_size per_frames align used_object total_object
pub fn print_slab_system_info() {
    let cache_list = unsafe { &SLAB_CACHES };
    println!("There are {} caches in system:", cache_list.len());
    println!("cache_name object_size per_frames align used_object total_object");
    cache_list.iter().for_each(|cache|{
        let cache = unsafe { &(*container_of!(cache as usize, MemCache, list)) };
        println!("---------------------------------------------------------------");
        cache.print_info();
    });
}

/// 分配一个指定大小和对齐方式的内存
/// 这里暂时忽略了对齐带来的影响
pub fn alloc_from_slab(size:usize,_align:usize) -> Option<*mut u8> {
    // 遍历所有的slab，找到第一个能够分配的slab
    let cache_list = unsafe{&mut SLAB_CACHES};
    let cache = cache_list.iter().find(|&x|{
        let cache = unsafe{&*container_of!(x as usize,MemCache,list)};
        cache.object_size as usize >= size
    });
    if cache.is_none(){
        return None;
    }else {
        let cache = unsafe{&mut *container_of!(cache.unwrap() as usize,MemCache,list)};
        let addr = cache.alloc();
        Some(addr)
    }
}

pub fn dealloc_to_slab(addr:*mut u8){
    let cache_list = unsafe { &SLAB_CACHES };
    cache_list.iter().for_each(|cache|{
        let cache = unsafe { &mut (*container_of!(cache as usize, MemCache, list)) };
        cache.dealloc(addr);
    });
}


/// initial the first cache
pub fn mem_cache_init() {
    unsafe {
        list_head_init!(SLAB_CACHES);
    }
    let cache = unsafe { &mut MEM_CACHE_BOOT };
    cache.init(
        b"kmem_cache",
        core::mem::size_of::<MemCache>() as u32,
        0,
        core::mem::align_of::<MemCache>() as u32,
    );
    list_add_tail!(
        to_list_head_ptr!(cache.list),
        to_list_head_ptr!(SLAB_CACHES)
    );
}



pub fn test_slab_system(){
    let cache = unsafe { &mut MEM_CACHE_BOOT };
    let test_addr = cache.alloc();
    info!("after alloc first object at:{:?}",test_addr);
    info!("{:?}", cache);
    cache.dealloc(test_addr);
    info!("after dealloc first object at:{:?}", test_addr);
    info!("{:?}", cache);
    for _ in 0..cache.per_objects {
        let _ = cache.alloc();
    }
    info!("after alloc all objects:");
    info!("{:?}", cache);
    for i in 0..cache.per_objects as usize {
        unsafe {
            cache.dealloc(test_addr.add( i * cache.object_size as usize));
        }
    }
    info!("after dealloc all objects:\n{:?}", cache);
    print_slab_system_info();
    reclaim_frame_from_cache(|start,count|{
        info!("reclaim frame from cache at:{:x} count:{}",start,count);
    });
    print_slab_system_info();
}



#[cfg(test)]
mod test {
    #[test]
    fn test_slab_alloc() {}
    #[test]
    fn test_slab_dealloc() {}
}
