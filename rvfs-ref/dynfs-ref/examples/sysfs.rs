use std::{
    error::Error,
    sync::{atomic::AtomicUsize, Arc, Weak},
};

use dynfs::{DynFs, DynFsDirInode, DynFsKernelProvider};
use dynfs_ref as dynfs;
use log::info;
use spin::{Mutex, Once};
use vfscore::{
    dentry::VfsDentry, error::VfsError, file::VfsFile, fstype::VfsFsType, inode::VfsInode,
    path::DirIter, utils::*, DVec, VfsResult,
};

#[derive(Clone)]
struct DynFsKernelProviderImpl;

impl DynFsKernelProvider for DynFsKernelProviderImpl {
    fn current_time(&self) -> VfsTimeSpec {
        VfsTimeSpec::new(0, 0)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    fake_rref::fake_init_rref();
    let pipefs = Arc::new(DynFs::<_, Mutex<()>>::new(
        DynFsKernelProviderImpl,
        "pipefs",
    ));
    println!("pipefs fs_name: {}", pipefs.fs_name());
    init_pipefs(pipefs.clone());
    let (reader, writer) = make_pipe_file()?;
    println!(
        "reader: {:?}, r:{}, w:{}",
        reader.dentry().name(),
        reader.is_readable(),
        reader.is_writable()
    );
    println!(
        "writer: {:?}, r:{}, w:{}",
        writer.dentry().name(),
        writer.is_readable(),
        writer.is_writable()
    ); // same "0"

    let root = PIPE_FS_ROOT.get().unwrap();
    root.inode()?.children().for_each(|x| {
        println!("{:?}", x.name);
    });
    // drop writer
    // drop reader

    Ok(())
}

type PipeFsDirInodeImpl = DynFsDirInode<DynFsKernelProviderImpl, Mutex<()>>;
static PIPE_FS_ROOT: Once<Arc<dyn VfsDentry>> = Once::new();

static PIPE: AtomicUsize = AtomicUsize::new(0);

/// 管道文件
pub struct PipeFile {
    readable: bool,
    writable: bool,
    dentry: Arc<dyn VfsDentry>,
    inode_copy: Arc<PipeInode>,
}

impl PipeFile {
    pub fn new(
        dentry: Arc<dyn VfsDentry>,
        readable: bool,
        writable: bool,
        inode_copy: Arc<PipeInode>,
    ) -> Self {
        Self {
            readable,
            writable,
            dentry,
            inode_copy,
        }
    }
}

pub fn init_pipefs(fs: Arc<dyn VfsFsType>) {
    let root = fs.i_mount(0, "", None, &[]).unwrap();
    PIPE_FS_ROOT.call_once(|| root);
}

/// create a pipe file
pub fn make_pipe_file() -> VfsResult<(Arc<PipeFile>, Arc<PipeFile>)> {
    let root = PIPE_FS_ROOT.get().unwrap();
    let root_inode = root
        .inode()?
        .downcast_arc::<PipeFsDirInodeImpl>()
        .map_err(|_| VfsError::Invalid)
        .unwrap();
    let inode = Arc::new(PipeInode::new());
    let num_str = PIPE.fetch_add(1, core::sync::atomic::Ordering::AcqRel);
    let same_inode =
        root_inode.add_file_manually(&num_str.to_string(), inode.clone(), "rw-rw-rw-".into())?;
    let dt = root.i_insert(&num_str.to_string(), same_inode)?;

    let reader = Arc::new(PipeFile::new(dt.clone(), true, false, inode.clone()));
    let sender = Arc::new(PipeFile::new(dt, false, true, inode.clone()));
    inode.set_reader(&reader);
    inode.set_sender(&sender);
    Ok((reader, sender))
}

impl PipeFile {
    fn dentry(&self) -> Arc<dyn VfsDentry> {
        self.dentry.clone()
    }

    fn is_readable(&self) -> bool {
        self.readable
    }

    fn is_writable(&self) -> bool {
        self.writable
    }
}

/// 环形缓冲区，用于在内存中维护管道的相关信息。
pub struct PipeInode {
    data: Mutex<PipeInodeData>,
}

const PIPE_BUF: usize = 512;
struct PipeInodeData {
    #[allow(unused)]
    /// 缓冲区的数据部分
    pub buf: [u8; PIPE_BUF],
    /// 缓冲区头部，用于指明当前的读位置
    pub head: usize,
    /// 缓冲区尾部，用于指明当前的写位置
    pub tail: usize,
    /// 记录 在 读端 进行等待的进程
    pub read_wait: Option<Weak<PipeFile>>,
    /// 记录 在 写端 进行等待的进程
    pub write_wait: Option<Weak<PipeFile>>,
}

impl PipeInodeData {
    /// 返回当前缓冲区中能够被读的字节数
    pub fn available_read(&self) -> usize {
        if self.head <= self.tail {
            self.tail - self.head
        } else {
            PIPE_BUF - self.head + self.tail
        }
    }
    /// 返回当前缓冲区中还能够写入的字节数
    pub fn available_write(&self) -> usize {
        if self.head <= self.tail {
            PIPE_BUF - self.tail + self.head - 1
        } else {
            self.head - self.tail - 1
        }
    }

    /// 返回是否有进程在 写端等待
    pub fn is_write_wait(&self) -> bool {
        self.write_wait.is_some() && self.write_wait.as_ref().unwrap().upgrade().is_some()
    }

    /// 返回是否有进程在 读端等待
    pub fn is_read_wait(&self) -> bool {
        self.read_wait.is_some() && self.read_wait.as_ref().unwrap().upgrade().is_some()
    }
}

impl Default for PipeInode {
    fn default() -> Self {
        Self::new()
    }
}

impl PipeInode {
    /// 创建一片新的管道缓冲区，在 `Pipe::new` 中被调用
    pub fn new() -> PipeInode {
        PipeInode {
            data: Mutex::new(PipeInodeData {
                buf: [0; PIPE_BUF],
                head: 0,
                tail: 0,
                read_wait: None,
                write_wait: None,
            }),
        }
    }

    pub fn set_reader(&self, reader: &Arc<PipeFile>) {
        let mut data = self.data.lock();
        data.read_wait = Some(Arc::downgrade(reader))
    }
    pub fn set_sender(&self, sender: &Arc<PipeFile>) {
        let mut data = self.data.lock();
        data.write_wait = Some(Arc::downgrade(sender))
    }
}

impl VfsFile for PipeInode {
    fn read_at(&self, _offset: u64, buf: DVec<u8>) -> VfsResult<(DVec<u8>, usize)> {
        Ok((buf, 0))
    }
    fn write_at(&self, _offset: u64, _buf: &DVec<u8>) -> VfsResult<usize> {
        Ok(0)
    }
    fn poll(&self, event: VfsPollEvents) -> VfsResult<VfsPollEvents> {
        let data = self.data.lock();
        let mut res = VfsPollEvents::empty();
        if event.contains(VfsPollEvents::IN) && data.available_read() > 0 {
            res |= VfsPollEvents::IN;
        }
        if event.contains(VfsPollEvents::OUT) && data.available_write() > 0 {
            res |= VfsPollEvents::OUT
        }
        let is_reader = data.is_read_wait();
        let is_sender = data.is_write_wait();
        if is_reader && !is_sender {
            res |= VfsPollEvents::HUP;
        }
        if !is_reader && is_sender {
            res |= VfsPollEvents::ERR;
        }
        Ok(res)
    }
}

impl VfsInode for PipeInode {
    fn node_perm(&self) -> VfsNodePerm {
        VfsNodePerm::empty()
    }
    fn inode_type(&self) -> VfsNodeType {
        VfsNodeType::Fifo
    }
}

impl Drop for PipeFile {
    fn drop(&mut self) {
        let data = self.inode_copy.data.lock();
        let is_reader = data.is_read_wait();
        let is_sender = data.is_write_wait();
        info!("is_reader: {}, is_sender: {}", is_reader, is_sender);
        if !is_reader && !is_sender {
            info!("remove pipe file manually");
            let name = self.dentry.name();
            let root = PIPE_FS_ROOT.get().unwrap();
            let root_inode = root
                .inode()
                .unwrap()
                .downcast_arc::<PipeFsDirInodeImpl>()
                .map_err(|_| VfsError::Invalid)
                .unwrap();
            root.remove(&name).unwrap();
            root_inode.remove_manually(&name).unwrap();
        } else {
            info!("pipe file is not removed manually");
        }
    }
}
