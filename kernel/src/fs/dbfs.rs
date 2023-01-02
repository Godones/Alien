//! DBFS
//!
//! 使用jammdb数据库构建文件系统,jammdb的底层文件依赖需要
//! 将一个存储设备模拟为一个文件,并且需要支持mmap
use crate::driver::QEMU_BLOCK_DEVICE;
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;
use alloc::{format, vec};
use core::alloc::Layout;
use core::cmp::min;
use core::fmt::{Debug, Display, Formatter};
use core::num::NonZeroUsize;
use core::ops::Deref;
use core2::io::{Read, Seek, SeekFrom, Write};
use fat32::BlockDevice;
use fat32_trait::{DirectoryLike, FileLike};
use jammdb::{Data, DbFile, FileExt, MemoryMap, MetaData, OpenOption, PathLike, DB};
use lru::LruCache;

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

impl FileExt for FakeFile {
    fn open<T: PathLike + ToString>(path: &T) -> Option<Self>
    where
        Self: Sized,
    {
        let device = QEMU_BLOCK_DEVICE.lock();
        let device = device.as_ref().unwrap();
        let mut file = FakeFile::new(device.clone());
        Some(file)
    }

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
        false
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
            FAKE_MMAP.size = size;
            FAKE_MMAP.addr = addr as usize;
        }
        Ok(unsafe { FAKE_MMAP.clone() })
    }
}

// impl Drop for FakeMMap {
//     fn drop(&mut self) {
//         info!("drop mmap");
//         let layout = Layout::from_size_align(self.size, 8).unwrap();
//         unsafe {
//             alloc::alloc::dealloc(self.addr as *mut u8, layout);
//         }
//     }
// }

pub struct SafeDb(DB);
unsafe impl Sync for SafeDb {}
unsafe impl Send for SafeDb {}

pub struct DbFileSystem {
    db: Arc<SafeDb>,
}
pub struct File {
    db: Arc<SafeDb>,
    name: String,
    dir: Arc<DirEntry>,
}

impl File {
    pub fn new(db: Arc<SafeDb>, name: &str, dir: Arc<DirEntry>) -> Self {
        Self {
            db,
            name: name.to_string(),
            dir,
        }
    }
}
impl Debug for File {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("File")
            .field("name", &self.name)
            .field("dir", &self.dir)
            .finish()
    }
}

pub struct DirEntry {
    db: Arc<SafeDb>,
    path: String,
}

impl DirEntry {
    pub fn new(db: Arc<SafeDb>, path: String) -> Self {
        Self { db, path }
    }
}

impl Debug for DirEntry {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("DirEntry")
            .field("path", &self.path)
            .finish()
    }
}

impl FileLike for File {
    type Error = Error;

    fn read(&self, offset: u32, size: u32) -> Result<Vec<u8>, Self::Error> {
        let tx = self.db.0.tx(false)?;
        let bucket = tx.get_bucket(self.dir.path.as_str())?;

        let bucket = bucket.get_bucket(self.name.as_str())?;
        // where is the offset?
        let data = bucket.get("data").unwrap();
        let data = data.kv().value().to_vec();

        info!("read data len:{}", data.len());
        // read data
        if (data.len() as u32) < offset {
            Ok(vec![])
        } else {
            let end = min(offset + size, data.len() as u32);
            Ok(data[offset as usize..end as usize].to_vec())
        }
    }

    fn write(&self, offset: u32, w_data: &[u8]) -> Result<u32, Self::Error> {
        let tx = self.db.0.tx(true)?;
        let bucket = tx.get_bucket(self.dir.path.as_str())?;
        let bucket = bucket.get_bucket(self.name.as_str())?;
        let data = bucket.get("data").unwrap();
        let mut data = data.kv().value().to_vec();
        if (data.len() as u32) < offset {
            data.resize(offset as usize, 0);
            for _ in data.len()..offset as usize {
                data.push(0);
            }
        }

        info!("data len:{}", data.len());
        data.extend_from_slice(&w_data);

        info!("extended data len: {}", data.len());
        bucket.put("data", data).unwrap();
        tx.commit().unwrap();
        Ok(w_data.len() as u32)
    }

    fn clear(&self) {
        let tx = self.db.0.tx(true).unwrap();
        let bucket = tx.get_bucket(self.dir.path.as_str()).unwrap();
        let bucket = bucket.get_bucket(self.name.as_str()).unwrap();
        bucket.put("data", vec![]).unwrap();
        tx.commit().unwrap();
    }

    fn size(&self) -> u32 {
        let tx = self.db.0.tx(false).unwrap();
        let bucket = tx.get_bucket(self.dir.path.as_str()).unwrap();
        let bucket = bucket.get_bucket(self.name.as_str()).unwrap();
        let data = bucket.get("data").unwrap();
        let data = data.kv().value().len();
        data as u32
    }
}

impl DirectoryLike for DirEntry {
    type Error = Error;

    fn create_dir(&self, name: &str) -> Result<(), Self::Error> {
        let tx = self.db.0.tx(true)?;
        let bucket = tx.get_bucket(self.path.as_str())?;
        bucket.create_bucket(name).unwrap();
        tx.commit().unwrap();
        Ok(())
    }

    fn create_file(&self, name: &str) -> Result<(), Self::Error> {
        info!("create file:{} in {}", name, self.path);
        let tx = self.db.0.tx(true)?;
        // x.into_iter().for_each(|x|info!("bucket:{}",core::str::from_utf8(x.0.name()).unwrap()));
        let bucket = tx.get_bucket(self.path.as_str()).unwrap();
        let bucket = bucket.create_bucket(name).unwrap();
        bucket.put("data", vec![]).unwrap();
        tx.commit().unwrap();
        Ok(())
    }

    fn delete_dir(&self, name: &str) -> Result<(), Self::Error> {
        let tx = self.db.0.tx(true)?;
        let bucket = tx.get_bucket(self.path.as_str())?;
        bucket.delete_bucket(name).unwrap();
        tx.commit().unwrap();
        Ok(())
    }

    fn delete_file(&self, name: &str) -> Result<(), Self::Error> {
        let tx = self.db.0.tx(true)?;
        let bucket = tx.get_bucket(self.path.as_str())?;
        bucket.delete_bucket(name)?;
        tx.commit().unwrap();
        Ok(())
    }

    fn cd(&self, name: &str) -> Result<Arc<dyn DirectoryLike<Error = Self::Error>>, Self::Error> {
        let tx = self.db.0.tx(false)?;
        let bucket = tx.get_bucket(self.path.as_str())?;
        let _bucket = bucket.get_bucket(name)?;
        Ok(Arc::new(DirEntry::new(
            self.db.clone(),
            format!("{}/{}", self.path, name),
        )))
    }

    fn open(&self, name: &str) -> Result<Arc<dyn FileLike<Error = Self::Error>>, Self::Error> {
        let tx = self.db.0.tx(false)?;
        let bucket = tx.get_bucket(self.path.as_str())?;
        let _bucket = bucket.get_bucket(name)?;
        let dir = Arc::new(DirEntry::new(self.db.clone(), self.path.clone()));
        Ok(Arc::new(File::new(self.db.clone(), name, dir)))
    }

    fn list(&self) -> Result<Vec<String>, Self::Error> {
        let tx = self.db.0.tx(false)?;
        let bucket = tx.get_bucket(self.path.as_str())?;
        let mut list = Vec::new();
        bucket.cursor().into_iter().for_each(|data| {
            let name = match &*data {
                Data::Bucket(b) => b.name(),
                Data::KeyValue(kv) => kv.key(),
            };
            list.push(String::from_utf8(name.to_vec()).unwrap());
        });
        Ok(list)
    }

    fn rename_file(&self, old_name: &str, new_name: &str) -> Result<(), Self::Error> {
        if old_name == new_name {
            return Ok(());
        }
        let tx = self.db.0.tx(true)?;
        let r_bucket = tx.get_bucket(self.path.as_str())?;
        let bucket = r_bucket.get_bucket(old_name).unwrap();
        let data = bucket.get("data").unwrap();
        let data = data.kv().value().to_vec();
        r_bucket.delete_bucket(old_name).unwrap();
        r_bucket.create_bucket(new_name).unwrap();
        let bucket = r_bucket.get_bucket(new_name).unwrap();
        bucket.put("data", data).unwrap();
        tx.commit().unwrap();
        Ok(())
    }

    fn rename_dir(&self, old_name: &str, new_name: &str) -> Result<(), Self::Error> {
        self.rename_file(old_name, new_name)
    }
}

#[derive(Debug)]
pub enum Error {
    NotFound,
    NotADir,
    NotAFile,
    NotEmpty,
    DBError(jammdb::Error),
    AlreadyExists,
    Other,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("{:?}", self))
    }
}

impl core::error::Error for Error {}

impl From<jammdb::Error> for Error {
    fn from(value: jammdb::Error) -> Self {
        Error::DBError(value)
    }
}

impl DbFileSystem {
    pub fn init() -> Self {
        let db = DB::open::<_, FakeOpenOptions, FakeMMap>(FakePath::new("sss")).unwrap();
        Self {
            db: Arc::new(SafeDb(db)),
        }
    }
    pub fn root(&self) -> Arc<DirEntry> {
        // 检查根目录是否存在
        let tx = self.db.0.tx(true).unwrap();
        info!("check root exist");
        let root = tx.get_or_create_bucket("root");
        assert!(root.is_ok());
        tx.commit().unwrap();
        let tx = self.db.0.tx(false).unwrap();
        let root = tx.get_bucket("root");
        if root.is_err() {
            panic!("root not exist");
        }
        Arc::new(DirEntry::new(self.db.clone(), "root".to_string()))
    }
}

#[allow(unused)]
pub fn jammdb_test() {
    let db = DB::open::<_, FakeOpenOptions, FakeMMap>(FakePath::new("sss")).unwrap();
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
        println!("{}:{}", key, value);
    }
}

#[allow(unused)]
pub fn dbfs_test() {
    let fs = DbFileSystem::init();
    let root = fs.root();
    println!("{:?}", root);
    for i in 0..10 {
        root.create_file(format!("file{}", i).as_str()).unwrap();
    }

    root.list().iter().for_each(|x| println!("{:?}", x));
    let file = root.open("file1").unwrap();
    println!("{:?}", file);

    file.write(0, b"hello world").unwrap();
    let data = file.read(0, 20).unwrap();
    println!("data: {}", String::from_utf8(data).unwrap());

    file.write(20, b"hello world").unwrap();
    let data = file.read(0, 31).unwrap();
    println!("data size: {}", data.len());
    println!("data: {}", String::from_utf8(data).unwrap());

    for i in 0..10 {
        root.rename_file(
            format!("file{}", i).as_str(),
            format!("new_file{}", i).as_str(),
        )
        .unwrap();
    }
    let new_file = root.open("new_file1").unwrap();
    let size = new_file.size();
    println!("file size: {}", size);

    root.list().iter().for_each(|x| println!("{:?}", x));

    for i in 0..9 {
        root.delete_file(format!("new_file{}", i).as_str()).unwrap();
    }
    root.list().iter().for_each(|x| println!("{:?}", x));
}
