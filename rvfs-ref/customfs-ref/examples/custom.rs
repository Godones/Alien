use std::{cmp::min, sync::Arc};

use custom_fs::{CustomFs, FsKernelProvider};
use custom_fs_ref as custom_fs;
use spin::Mutex;
use vfscore::{
    error::VfsError,
    file::VfsFile,
    fstype::VfsFsType,
    inode::VfsInode,
    path::print_fs_tree,
    utils::{VfsDirEntry, VfsFileStat, VfsNodeType, VfsTimeSpec},
    DVec, VfsResult,
};

#[derive(Clone)]
struct FsKernelProviderImpl;

impl FsKernelProvider for FsKernelProviderImpl {
    fn current_time(&self) -> VfsTimeSpec {
        VfsTimeSpec::new(0, 0)
    }
}
fn main() {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("error"));
    fake_rref::fake_init_rref();
    let fs = Arc::new(CustomFs::<_, Mutex<()>>::new(
        FsKernelProviderImpl,
        "customfs",
        Arc::new(CustomRootInode::new()),
    ));
    let root = fs.mount(0, "/", None, &[]).unwrap();
    let root_inode = root.inode().unwrap();
    let root_inode = root_inode
        .downcast_arc::<CustomRootInode>()
        .map_err(|_| "root inode is not a CustomRootInode")
        .unwrap();

    for i in 0..5 {
        let name = format!("file{}", i);
        let data = format!("data{}", i);
        let inode = Arc::new(CustomInode::new(data.as_bytes()));
        root_inode.insert_inode(name, inode);
    }

    root_inode.insert_inode("dir1".to_string(), Arc::new(CustomRootInode::new()));
    root_inode.insert_inode("dir2".to_string(), Arc::new(CustomRootInode::new()));

    let dir1 = root_inode.lookup("dir1").unwrap();
    let dir1_inode = dir1
        .downcast_arc::<CustomRootInode>()
        .map_err(|_| "dir1 inode is not a CustomRootInode")
        .unwrap();
    for i in 0..5 {
        let name = format!("f{}", i);
        let data = format!("data{}", i);
        let inode = Arc::new(CustomInode::new(data.as_bytes()));
        dir1_inode.insert_inode(name, inode);
    }

    let root_inode = root.inode().unwrap();
    println!("root inode type: {:?}", root_inode.inode_type());
    print_fs_tree(&mut FakeWriter, root, "".to_string(), true).unwrap();
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
pub enum DomainTypeRaw {
    FsDomain = 1,
    BlkDeviceDomain = 2,
    CacheBlkDeviceDomain = 3,
    RtcDomain = 4,
    GpuDomain = 5,
    InputDomain = 6,
    VfsDomain = 7,
    UartDomain = 8,
    PLICDomain = 9,
    TaskDomain = 10,
    SysCallDomain = 11,
    ShadowBlockDomain = 12,
    BufUartDomain = 13,
    NetDeviceDomain = 14,
    BufInputDomain = 15,
    EmptyDeviceDomain = 16,
    DevFsDomain = 17,
    SchedulerDomain = 18,
    LogDomain = 19,
    NetDomain = 20,
}
struct FakeWriter;

impl core::fmt::Write for FakeWriter {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        print!("{}", s);
        Ok(())
    }
}

#[derive(Default)]
struct CustomRootInode {
    children: Mutex<Vec<(String, Arc<dyn VfsInode>)>>,
}

impl CustomRootInode {
    pub fn new() -> Self {
        Self {
            children: Mutex::new(Vec::new()),
        }
    }
    pub fn insert_inode(&self, name: String, inode: Arc<dyn VfsInode>) {
        if self.children.lock().iter().find(|x| x.0 == name).is_none() {
            self.children.lock().push((name.to_string(), inode));
        }
    }
}

impl VfsFile for CustomRootInode {
    fn readdir(&self, start_index: usize) -> VfsResult<Option<VfsDirEntry>> {
        let children = self.children.lock();
        if start_index >= children.len() {
            return Ok(None);
        }
        let (name, inode) = &children[start_index];
        Ok(Some(VfsDirEntry {
            ino: 0,
            ty: inode.inode_type(),
            name: name.clone(),
        }))
    }
}

impl VfsInode for CustomRootInode {
    fn lookup(&self, name: &str) -> VfsResult<Arc<dyn VfsInode>> {
        let res = self
            .children
            .lock()
            .iter()
            .find(|(f_name, _)| f_name == name)
            .map(|(_, inode)| inode.clone());
        match res {
            Some(inode) => Ok(inode),
            None => Err(VfsError::NoEntry),
        }
    }
    fn get_attr(&self) -> VfsResult<VfsFileStat> {
        Ok(VfsFileStat {
            st_dev: 0,
            st_ino: 0,
            st_mode: 0o644,
            st_nlink: 1,
            st_uid: 0,
            st_gid: 0,
            st_rdev: 0,
            __pad: 0,
            st_size: 4096,
            st_blksize: 512,
            __pad2: 0,
            st_blocks: 0,
            st_atime: VfsTimeSpec::default(),
            st_mtime: VfsTimeSpec::default(),
            st_ctime: VfsTimeSpec::default(),
            unused: 0,
        })
    }
    fn inode_type(&self) -> VfsNodeType {
        VfsNodeType::Dir
    }
}

struct CustomInode {
    data: Vec<u8>,
}

impl CustomInode {
    pub fn new(data: &[u8]) -> Self {
        Self {
            data: data.to_vec(),
        }
    }
}

impl VfsFile for CustomInode {
    fn read_at(&self, offset: u64, buf: DVec<u8>) -> VfsResult<(DVec<u8>, usize)> {
        if offset as usize >= self.data.len() {
            return Ok((buf, 0));
        }
        let copy_start = min(offset as usize, self.data.len());
        let copy_end = min(copy_start + buf.len(), self.data.len());
        let copied = copy_end - copy_start;
        let mut buf = buf;
        buf.as_mut_slice()[..copied].copy_from_slice(&self.data[copy_start..copy_end]);
        Ok((buf, copied))
    }
}

impl VfsInode for CustomInode {
    fn get_attr(&self) -> VfsResult<VfsFileStat> {
        Ok(VfsFileStat {
            st_dev: 0,
            st_ino: 0,
            st_mode: 0o644,
            st_nlink: 1,
            st_uid: 0,
            st_gid: 0,
            st_rdev: 0,
            __pad: 0,
            st_size: self.data.len() as u64,
            st_blksize: 512,
            __pad2: 0,
            st_blocks: 0,
            st_atime: VfsTimeSpec::default(),
            st_mtime: VfsTimeSpec::default(),
            st_ctime: VfsTimeSpec::default(),
            unused: 0,
        })
    }
    fn inode_type(&self) -> VfsNodeType {
        VfsNodeType::File
    }
}
