use crate::config::FRAME_SIZE;
use crate::driver::{QemuBlockDevice, QEMU_BLOCK_DEVICE};
use crate::memory::{frame_alloc, FrameTracker};
use crate::task::{current_process, Process};
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use core::cmp::min;
use core::fmt::{Display, Formatter};
use core::num::NonZeroUsize;
use core2::io::{Read, Seek, SeekFrom, Write};
use dbop::{Operate, OperateSet};
use jammdb::{
    DbFile, File, FileExt, IOResult, IndexByPageID, MemoryMap, MetaData, Mmap, OpenOption,
    PathLike, DB,
};
use lru::LruCache;
use rvfs::superblock::Device;
use spin::{Mutex, Once};
use syscall_table::syscall_func;

static CACHE_LAYER: Once<Mutex<CacheLayer>> = Once::new();
const PAGE_CACHE_SIZE: usize = FRAME_SIZE;

pub struct CacheLayer {
    device: Arc<QemuBlockDevice>,
    lru: LruCache<usize, FrameTracker>,
}
impl CacheLayer {
    pub fn new(device: Arc<QemuBlockDevice>, limit: usize) -> Self {
        Self {
            device,
            lru: LruCache::new(NonZeroUsize::new(limit).unwrap()),
        }
    }
    pub fn get(&mut self, id: usize) -> Option<&FrameTracker> {
        let flag = self.lru.contains(&id);
        if flag {
            self.lru.get(&id)
        } else {
            let mut cache = frame_alloc().unwrap();
            let start = id * PAGE_CACHE_SIZE;
            let start_block = start / 512;
            let end_block = (start + PAGE_CACHE_SIZE) / 512;
            for i in start_block..end_block {
                let target_buf = &mut cache[(i - start_block) * 512..(i - start_block + 1) * 512];
                self.device.device.lock().read_block(i, target_buf).unwrap();
            }
            let old = self.lru.push(id, cache);
            // write back
            if let Some((id, frame)) = old {
                warn!("write back frame {} to disk", id);
                let start = id * PAGE_CACHE_SIZE;
                let start_block = start / 512;
                let end_block = (start + PAGE_CACHE_SIZE) / 512;
                for i in start_block..end_block {
                    let target_buf = &frame[(i - start_block) * 512..(i - start_block + 1) * 512];
                    self.device
                        .device
                        .lock()
                        .write_block(i, target_buf)
                        .unwrap();
                }
            }
            let cache = self.lru.get(&id);
            cache
        }
    }
    pub fn get_mut(&mut self, id: usize) -> Option<&mut FrameTracker> {
        let flag = self.lru.contains(&id);
        if flag {
            self.lru.get_mut(&id)
        } else {
            let mut cache = frame_alloc().unwrap();
            let start = id * PAGE_CACHE_SIZE;
            let start_block = start / 512;
            let end_block = (start + PAGE_CACHE_SIZE) / 512;
            for i in start_block..end_block {
                let target_buf = &mut cache[(i - start_block) * 512..(i - start_block + 1) * 512];
                self.device.device.lock().read_block(i, target_buf).unwrap();
            }
            let old = self.lru.push(id, cache);
            // write back
            if let Some((id, frame)) = old {
                warn!("write back frame {} to disk", id);
                let start = id * PAGE_CACHE_SIZE;
                let start_block = start / 512;
                let end_block = (start + PAGE_CACHE_SIZE) / 512;
                for i in start_block..end_block {
                    let target_buf = &frame[(i - start_block) * 512..(i - start_block + 1) * 512];
                    self.device
                        .device
                        .lock()
                        .write_block(i, target_buf)
                        .unwrap();
                }
            }
            let cache = self.lru.get_mut(&id);
            cache
        }
    }
    pub fn flush(&self) {
        for (id, frame) in self.lru.iter() {
            let start = id * PAGE_CACHE_SIZE;
            let start_block = start / 512;
            let end_block = (start + PAGE_CACHE_SIZE) / 512;
            for i in start_block..end_block {
                let target_buf = &frame[(i - start_block) * 512..(i - start_block + 1) * 512];
                self.device
                    .device
                    .lock()
                    .write_block(i, target_buf)
                    .unwrap();
            }
        }
    }
}

/// we use a fake file to represent the block device
///
/// The first page is used to store the size of the file, the pagesize is 4KB.
pub struct FakeFile {
    offset: usize,
    size: usize,
    device: Arc<QemuBlockDevice>,
}

impl FakeFile {
    fn new(device: Arc<QemuBlockDevice>) -> Self {
        let mut buf = [0u8; 512];
        device.device.lock().read_block(0, &mut buf).unwrap();
        let size = usize::from_le_bytes(buf[0..8].try_into().unwrap());
        CACHE_LAYER.call_once(|| Mutex::new(CacheLayer::new(device.clone(), 1024))); // 1024 * 4096 = 4MB
        Self {
            offset: 0,
            size,
            device,
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
                    return Err(core2::io::Error::new(
                        core2::io::ErrorKind::InvalidInput,
                        "invalid seek to a negative or overflowing position",
                    ));
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
            let cache_id = offset / PAGE_CACHE_SIZE;
            let cache_offset = offset % PAGE_CACHE_SIZE;
            // we need write from cache_offset to 4KB
            let mut layer = CACHE_LAYER.get().unwrap().lock();
            let cache = layer.get_mut(cache_id + 1).unwrap();
            // let cache = self.cache_manager.get_mut(cache_id + 1).unwrap();
            let n = min(PAGE_CACHE_SIZE - cache_offset, buf.len());
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

    /// write back the cache to the disk
    fn flush(&mut self) -> core2::io::Result<()> {
        // warn!("flush file");
        // CACHE_LAYER.get().unwrap().lock().flush();
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
            let cache_id = offset / PAGE_CACHE_SIZE;
            let cache_offset = offset % PAGE_CACHE_SIZE;
            // 文件从第一个块开始读取
            let mut layer = CACHE_LAYER.get().unwrap().lock();
            let cache = layer.get(cache_id + 1).unwrap();
            let n = min(PAGE_CACHE_SIZE - cache_offset, buf.len());
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

    /// write back the file size to the disk
    fn sync_all(&self) -> core2::io::Result<()> {
        // 因为块设备的第一块是用来存储文件系统的大小的
        // 所以需要将文件系统的大小写入到块设备的第一块
        // warn!("sync all metadata to disk");
        // let mut buf = [0u8; 512];
        // buf[0..8].copy_from_slice(&(self.size).to_le_bytes());
        // self.device.device.lock().write_block(0, &buf).unwrap();
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
        Ok(File::new(Box::new(fake_file.unwrap())))
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
pub struct FakeMMap;

impl MemoryMap for FakeMMap {
    fn map(&self, _file: &mut File) -> IOResult<Mmap>
    where
        Self: Sized,
    {
        let mmap = Mmap { size: 0, addr: 0 };
        Ok(mmap)
    }

    fn do_map(&self, _file: &mut File) -> IOResult<Arc<dyn IndexByPageID>> {
        let res = IndexByPageIDImpl;
        Ok(Arc::new(res))
    }
}

struct IndexByPageIDImpl;

impl IndexByPageID for IndexByPageIDImpl {
    fn index(&self, page_id: u64, page_size: usize) -> IOResult<&[u8]> {
        let mut layer = CACHE_LAYER.get().unwrap().lock();
        let cache = layer.get_mut(page_id as usize + 1).unwrap();
        let start = cache.start();
        unsafe { Ok(core::slice::from_raw_parts(start as *const u8, page_size)) }
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
