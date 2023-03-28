use crate::driver::QEMU_BLOCK_DEVICE;
use crate::task::{current_process, Process};
use alloc::alloc::dealloc;
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec;
use core::alloc::Layout;
use core::cmp::min;
use core::fmt::{Display, Formatter};
use core::num::NonZeroUsize;
use core::ops::Deref;
use core2::io::{Read, Seek, SeekFrom, Write};
use dbop::{Operate, OperateSet};
use jammdb::{DbFile, FileExt, MemoryMap, MetaData, Mmap, OpenOption, PathLike, DB};
use lru::LruCache;
use syscall_table::syscall_func;

type BlockDevice = dyn rvfs::superblock::Device;
type Cache = [u8; 512];
struct CacheManager {
    device: Arc<BlockDevice>,
    lru: LruCache<usize, Cache>,
}

impl CacheManager {
    pub fn new(device: Arc<BlockDevice>, limit: usize) -> Self {
        Self {
            device,
            lru: LruCache::new(NonZeroUsize::new(limit).unwrap()),
        }
    }
    pub fn get(&mut self, id: usize) -> Option<&Cache> {
        let flag = self.lru.contains(&id);
        if flag {
            self.lru.get(&id)
        } else {
            let mut cache = [0u8; 512];
            self.device.read(&mut cache, id * 512).unwrap();
            let old = self.lru.push(id, cache);
            let cache = self.lru.get(&id);
            if let Some((old_id, old_cache)) = old {
                self.device.write(&old_cache, old_id * 512).unwrap();
            }
            cache
        }
    }
    pub fn get_mut(&mut self, id: usize) -> Option<&mut Cache> {
        let flag = self.lru.contains(&id);
        if flag {
            self.lru.get_mut(&id)
        } else {
            let mut cache = [0u8; 512];
            self.device.read(&mut cache, id * 512).unwrap();
            let old = self.lru.push(id, cache);
            let cache = self.lru.get_mut(&id);
            if let Some((old_id, old_cache)) = old {
                self.device.write(&old_cache, old_id * 512).unwrap();
            }
            cache
        }
    }
    pub fn flush(&mut self) {
        self.lru.iter().for_each(|(id, cache)| {
            self.device.write(cache, id * 512).unwrap();
        })
    }
}

/// 将块设备模拟为一个文件
pub struct FakeFile {
    cache_manager: CacheManager,
    offset: usize,
    size: usize,
}

impl FakeFile {
    fn new(device: Arc<BlockDevice>) -> Self {
        let mut buf = [0u8; 512];
        device.read(&mut buf, 0).unwrap();
        let size = usize::from_le_bytes(buf[0..8].try_into().unwrap());
        Self {
            cache_manager: CacheManager::new(device, 1024),
            offset: 0,
            size,
        }
    }
}

impl Seek for FakeFile {
    fn seek(&mut self, pos: SeekFrom) -> core2::io::Result<u64> {
        info!("seek to {:?}", pos);
        match pos {
            SeekFrom::Start(offset) => {
                self.offset = offset as usize;
            }
            SeekFrom::Current(offset) => {
                self.offset += offset as usize;
            }
            SeekFrom::End(offset) => {
                self.offset = self.size + offset as usize;
                if (self.offset as isize) < 0 {
                    self.offset = 0;
                }
            }
        }
        Ok(self.offset as u64)
    }
}

impl Write for FakeFile {
    fn write(&mut self, buf: &[u8]) -> core2::io::Result<usize> {
        info!("[{}/{}] write buf len:{}", file!(), line!(), buf.len());
        let len = buf.len();
        let mut offset = self.offset;
        let mut buf = buf;
        while !buf.is_empty() {
            let cache_id = offset / 512;
            let cache_offset = offset % 512;
            // 文件从第一个块开始写入
            let cache = self.cache_manager.get_mut(cache_id + 1).unwrap();
            let n = min(512 - cache_offset, buf.len());
            cache[cache_offset..cache_offset + n].copy_from_slice(&buf[..n]);
            offset += n;
            buf = &buf[n..];
        }
        self.offset = offset;
        if self.offset > self.size {
            self.size = self.offset;
        }
        Ok(len)
    }

    fn flush(&mut self) -> core2::io::Result<()> {
        warn!("flush file");
        self.cache_manager.flush();
        // 因为块设备的第一块是用来存储文件系统的大小的
        // 所以需要将文件系统的大小写入到块设备的第一块
        // let mut buf = [0u8; 512];
        // buf[0..8].copy_from_slice(&(self.size as u64).to_le_bytes());
        // self.cache_manager.device.write(0, &buf).unwrap();
        let map_size = unsafe { FAKE_MMAP.size };
        let size = self.size();
        if map_size >= size {
            let mut data = vec![0u8; size];
            self.seek(SeekFrom::Start(0)).unwrap();
            self.read(data.as_mut_slice()).unwrap();
            unsafe {
                let addr = FAKE_MMAP.addr;
                core::ptr::copy(data.as_ptr(), addr as *mut u8, size);
            }
        }

        Ok(())
    }
}

impl Read for FakeFile {
    fn read(&mut self, buf: &mut [u8]) -> core2::io::Result<usize> {
        let len = buf.len();
        info!(
            "[{}/{}] read buf len:{:#x}, file offset:{}",
            file!(),
            line!(),
            buf.len(),
            self.offset
        );
        let mut offset = self.offset;
        let mut buf = buf;
        while !buf.is_empty() && offset < self.size {
            let cache_id = offset / 512;
            let cache_offset = offset % 512;
            // 文件从第一个块开始读取
            let cache = self.cache_manager.get(cache_id + 1).unwrap();
            let n = min(512 - cache_offset, buf.len());
            buf[..n].copy_from_slice(&cache[cache_offset..cache_offset + n]);
            offset += n;
            buf = &mut buf[n..];
        }
        self.offset = offset;
        Ok(len)
    }
}

impl FakeFile {
    fn open<T: PathLike + ToString>(_path: &T) -> Option<Self> {
        let device = QEMU_BLOCK_DEVICE.lock()[1].clone();
        let file = FakeFile::new(device);
        Some(file)
    }
}

impl FileExt for FakeFile {
    fn lock_exclusive(&self) -> core2::io::Result<()> {
        Ok(())
    }

    fn allocate(&mut self, new_size: u64) -> core2::io::Result<()> {
        info!(
            "[{}/{}] allocate new size:{:#x}",
            file!(),
            line!(),
            new_size
        );
        self.size = new_size as usize;
        // // 因为块设备的第一块是用来存储文件系统的大小的
        // // 所以需要将文件系统的大小写入到块设备的第一块
        // let mut buf = [0u8; 8];
        // buf[0..8].copy_from_slice(&(self.size as u64).to_le_bytes());
        // self.cache_manager.device.write(0, &buf).unwrap();
        Ok(())
    }

    fn unlock(&self) -> core2::io::Result<()> {
        Ok(())
    }

    fn metadata(&self) -> core2::io::Result<MetaData> {
        Ok(MetaData {
            len: self.size as u64,
        })
    }

    fn sync_all(&self) -> core2::io::Result<()> {
        // 因为块设备的第一块是用来存储文件系统的大小的
        // 所以需要将文件系统的大小写入到块设备的第一块
        warn!("sync all metadata");
        let mut buf = [0u8; 8];
        buf[0..8].copy_from_slice(&(self.size).to_le_bytes());
        self.cache_manager.device.write(&buf, 0).unwrap();

        Ok(())
    }

    fn size(&self) -> usize {
        self.size
    }

    fn addr(&self) -> usize {
        0
    }
}

impl DbFile for FakeFile {}

pub struct FakeOpenOptions;

impl OpenOption for FakeOpenOptions {
    fn new() -> Self {
        Self {}
    }

    fn read(&mut self, _read: bool) -> &mut Self {
        self
    }

    fn write(&mut self, _write: bool) -> &mut Self {
        self
    }

    fn open<T: ToString + PathLike>(&mut self, path: &T) -> core2::io::Result<jammdb::File> {
        info!("open file: {}", path.to_string());
        let fake_file = FakeFile::open(path);
        if fake_file.is_none() {
            return Err(core2::io::Error::new(
                core2::io::ErrorKind::NotFound,
                "file not found",
            ));
        }
        Ok(jammdb::File::new(Box::new(fake_file.unwrap())))
    }

    fn create(&mut self, _create: bool) -> &mut Self {
        self
    }
}
#[derive(Debug)]
struct FakePath {
    path: String,
}

impl Display for FakePath {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("FakePath({})", self.path))
    }
}

impl PathLike for FakePath {
    fn exists(&self) -> bool {
        let device = QEMU_BLOCK_DEVICE.lock()[1].clone();
        let mut buf = [0u8; 8];
        device.read(&mut buf, 0).unwrap();
        let size = usize::from_le_bytes(buf[0..8].try_into().unwrap());
        size > 0
    }
}
impl FakePath {
    #[allow(unused)]
    pub fn new(path: &str) -> Self {
        Self {
            path: path.to_string(),
        }
    }
}

#[derive(Clone, Default)]
pub struct FakeMMap {
    size: usize,
    addr: usize,
}

impl Deref for FakeMMap {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        unsafe { core::slice::from_raw_parts(self.addr as *const u8, self.size) }
    }
}

static mut FAKE_MMAP: Mmap = Mmap { size: 0, addr: 0 };

impl MemoryMap for FakeMMap {
    fn map(&self, file: &mut dyn DbFile) -> core2::io::Result<Mmap>
    where
        Self: Sized,
    {
        // 将文件映射到虚拟内存
        // 这里暂且使用物理内存
        let layout = Layout::from_size_align(file.size(), 8).unwrap();
        let addr = unsafe { alloc::alloc::alloc(layout) };
        let size = file.size();
        let mut data = vec![0u8; size];
        file.seek(SeekFrom::Start(0)).unwrap();
        file.read(data.as_mut_slice()).unwrap();
        unsafe {
            core::ptr::copy(data.as_ptr(), addr as *mut u8, size);
        }
        info!("[{}/{}] mmap file size:{:#x}", file!(), line!(), size);
        unsafe {
            let old_addr = FAKE_MMAP.addr;
            let old_size = FAKE_MMAP.size;
            if old_addr != 0 {
                let layout = Layout::from_size_align(old_size, 8).unwrap();
                dealloc(old_addr as *mut u8, layout);
            }
            FAKE_MMAP.size = size;
            FAKE_MMAP.addr = addr as usize;
        }
        let mmap = Mmap {
            size,
            addr: addr as usize,
        };
        Ok(mmap)
    }
}

fn init_db(db: &DB) {
    let tx = db.tx(true).unwrap();
    let bucket = tx.get_or_create_bucket("super_blk").unwrap();
    bucket.put("continue_number", 0usize.to_le_bytes()).unwrap();
    bucket.put("magic", 1111u32.to_le_bytes()).unwrap();
    bucket.put("blk_size", 512u32.to_le_bytes()).unwrap();
    tx.commit().unwrap()
}

pub fn init_dbfs() {
    let db = DB::open::<FakeOpenOptions, _>(Arc::new(FakeMMap::default()), "dbfs").unwrap();
    init_db(&db);
    dbfs2::init_dbfs(db);
}

#[syscall_func(1001)]
pub fn sys_create_global_bucket(key: *const u8) -> isize {
    let process = current_process().unwrap();
    let str = process.transfer_str(key);
    let res = dbfs2::extend::extend_create_global_bucket(&str);
    if res.is_ok() {
        0
    } else {
        0
    }
}

#[syscall_func(1002)]
pub fn sys_execute_user_func(key: *const u8, buf: *const u8, len: usize, func: usize) -> isize {
    let process = current_process().unwrap();
    let key = process.transfer_str(key);
    let mut buf = process.transfer_raw_buffer(buf, len);
    let func = process.transfer_raw(func);
    use dbfs2::extend::MyPara;
    let func = unsafe {
        core::mem::transmute::<*const (), fn(&str, MyPara, &mut [u8]) -> isize>(func as *const ())
    };

    warn!("will execute user func, the key is {:?}", key);
    dbfs2::extend::execute(&key, func, buf[0])
}

#[syscall_func(1003)]
pub fn sys_show_dbfs() -> isize {
    let res = dbfs2::extend::show_dbfs();
    if res.is_ok() {
        0
    } else {
        -1
    }
}

#[syscall_func(1004)]
pub fn sys_execute_user_operate(bucket: *const u8, operate: *const u8) -> isize {
    let process = current_process().unwrap();
    let bucket = process.transfer_str(bucket);
    let operate = process.transfer_str(operate);
    let mut operate: OperateSet = serde_json::from_str(&operate).unwrap();
    // we need modify the ReadOperate because it contains a pointer of buf
    operate.operate.iter_mut().for_each(|op| match op {
        Operate::Read(rop) => {
            let addr = rop.buf_addr;
            let new_addr = process.transfer_raw(addr);
            rop.buf_addr = new_addr;
        }
        Operate::AddBucket(_) | Operate::StepInto(_) => {
            update_buf_address_recursive(process, op);
        }
        _ => {}
    });

    let res = dbfs2::extend::execute_operate(&bucket, operate);
    res
}

fn update_buf_address_recursive(process: &Arc<Process>, operate: &mut Operate) {
    let local = |other: &mut Box<OperateSet>| {
        other.operate.iter_mut().for_each(|op| match op {
            Operate::Read(rop) => {
                let addr = rop.buf_addr;
                let new_addr = process.transfer_raw(addr);
                rop.buf_addr = new_addr;
            }
            Operate::AddBucket(_) | Operate::StepInto(_) => {
                update_buf_address_recursive(process, op);
            }
            _ => {}
        })
    };
    match operate {
        Operate::AddBucket(ab) => {
            if let Some(other) = &mut ab.other {
                local(other);
            }
        }
        Operate::StepInto(si) => {
            if let Some(other) = &mut si.other {
                local(other);
            }
        }
        _ => {}
    }
}
