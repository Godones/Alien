use alloc::alloc::dealloc;
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use lazy_static::lazy_static;
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
use core::alloc::Layout;
use core::cmp::min;
use core::num::NonZeroUsize;
use core::ops::Deref;
use core2::io::{Read, Seek, SeekFrom, Write};
use dbfs::{Dir,File,DbFileSystem};
use fat32_trait::BlockDevice;
use dbfs::jammdb::{DB, DbFile, FileExt, MemoryMap, MetaData, OpenOption, PathLike};
use lru::LruCache;
use crate::driver::QEMU_BLOCK_DEVICE;



lazy_static! {
    pub static ref ROOT_DIR: Arc<Dir> = {
        let db = DB::open::<_,FakeOpenOptions,FakeMMap>(FakePath::new("")).unwrap();
        let fs = DbFileSystem::new(db);
        let root = fs.root();
        root
    };
}



type Cache = [u8; 512];
struct CacheManager {
    device: Arc<dyn BlockDevice>,
    lru: LruCache<usize, Cache>,
    dirty: Vec<usize>,
}

impl CacheManager {
    pub fn new(device: Arc<dyn BlockDevice>, limit: usize) -> Self {
        Self {
            device,
            lru: LruCache::new(NonZeroUsize::new(limit).unwrap()),
            dirty: vec![],
        }
    }
    pub fn get(&mut self, id: usize) -> Option<&Cache> {
        let flag = self.lru.contains(&id);
        if flag {
            self.lru.get(&id)
        } else {
            let mut cache = [0u8; 512];
            self.device.read(id, &mut cache).unwrap();
            self.lru.put(id, cache);
            self.lru.get(&id)
        }
    }
    pub fn get_mut(&mut self, id: usize) -> Option<&mut Cache> {
        self.dirty.push(id);
        let flag = self.lru.contains(&id);
        if flag {
            self.lru.get_mut(&id)
        } else {
            let mut cache = [0u8; 512];
            self.device.read(id, &mut cache).unwrap();
            self.lru.put(id, cache);
            self.lru.get_mut(&id)
        }
    }
    pub fn flush(&mut self) {
        for id in self.dirty.iter() {
            let cache = self.lru.get(id).unwrap();
            self.device.write(*id, cache).unwrap();
        }
        self.dirty.clear();
    }
}

/// 将块设备模拟为一个文件
pub struct FakeFile {
    cache_manager: CacheManager,
    offset: usize,
    size: usize,
}

impl FakeFile {
    fn new(device: Arc<dyn BlockDevice>) -> Self {
        let mut buf = [0u8; 512];
        device.read(0, &mut buf).unwrap();
        let size = u64::from_le_bytes(buf[0..8].try_into().unwrap()) as usize;
        Self {
            cache_manager: CacheManager::new(device, 100),
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

impl FakeFile{
    fn open<T: PathLike + ToString>(_path: &T) -> Option<Self> {
        let device = QEMU_BLOCK_DEVICE.lock();
        let device = device.as_ref().unwrap();
        let file = FakeFile::new(device.clone());
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
        let mut buf = [0u8; 512];
        buf[0..8].copy_from_slice(&(self.size as u64).to_le_bytes());
        self.cache_manager.device.write(0, &buf).unwrap();

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

    fn open<T: ToString + PathLike>(&mut self, path: &T) -> core2::io::Result<dbfs::jammdb::File> {
        info!("open file: {}", path.to_string());
        let fake_file = FakeFile::open(path);
        if fake_file.is_none() {
            return Err(core2::io::Error::new(
                core2::io::ErrorKind::NotFound,
                "file not found",
            ));
        }
        Ok(dbfs::jammdb::File::new(Box::new(fake_file.unwrap())))
    }

    fn create(&mut self, _create: bool) -> &mut Self {
        self
    }
}

struct FakePath {
    path: String,
}

impl ToString for FakePath {
    fn to_string(&self) -> String {
        self.path.clone()
    }
}
impl PathLike for FakePath {
    fn exists(&self) -> bool {
        let device = QEMU_BLOCK_DEVICE.lock();
        let mut buf = [0u8; 512];
        let device = device.as_ref().unwrap();
        device.read(0, &mut buf).unwrap();
        let size = u64::from_le_bytes(buf[0..8].try_into().unwrap());
        size > 0
    }
}
impl FakePath {
    pub fn new(path: &str) -> Self {
        Self {
            path: path.to_string(),
        }
    }
}

#[derive(Clone)]
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

static mut FAKE_MMAP: FakeMMap = FakeMMap { size: 0, addr: 0 };

impl MemoryMap for FakeMMap {
    fn map(file: &mut dyn DbFile) -> core2::io::Result<Self>
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
        Ok(unsafe { FAKE_MMAP.clone() })
    }
}
#[allow(unused)]
pub fn jammdb_test() {
    let db = DB::open::<_, FakeOpenOptions, FakeMMap>(FakePath::new("")).unwrap();
    // // open a writable transaction so we can make changes
    let tx = db.tx(true).unwrap();
    // create a bucket to store a map of first names to last names
    let names_bucket = tx.create_bucket("names").unwrap();
    names_bucket.put(b"Kanan", b"Jarrus").unwrap();
    names_bucket.put(b"Ezra", b"Bridger").unwrap();
    // commit the changes so they are saved to disk
    tx.commit().unwrap();
    let tx = db.tx(false).unwrap();
    let users_bucket = tx.get_bucket("names").unwrap();
    // get the key / value pair we inserted into the bucket
    if let Some(data) = users_bucket.get(b"Kanan") {
        // deserialize into a user struct
        let key = core::str::from_utf8(data.kv().key()).unwrap();
        let value = core::str::from_utf8(data.kv().value()).unwrap();
        info!("{}:{}", key, value);
    }
}