#![no_std]
#![forbid(unsafe_code)]
extern crate alloc;
use alloc::{boxed::Box, string::ToString, sync::Arc};

use basic::sync::Mutex;
use dynfs::{DynFsDirInode, DynFsKernelProvider};
use generic::GenericFsDomain;
use interface::FsDomain;
use vfscore::{dentry::VfsDentry, error::VfsError, utils::VfsTimeSpec};

use crate::{
    filesystem::SystemSupportFS, interrupt::InterruptRecord, mem::MemInfo, mounts::MountInfo,
};

mod filesystem;
mod interrupt;
mod mem;
mod mounts;

type ProcFsDomain = GenericFsDomain;
type ProcFs = dynfs::DynFs<CommonFsProviderImpl, Mutex<()>>;
#[derive(Clone)]
pub struct CommonFsProviderImpl;

impl DynFsKernelProvider for CommonFsProviderImpl {
    fn current_time(&self) -> VfsTimeSpec {
        VfsTimeSpec::new(0, 0)
    }
}

pub fn main() -> Box<dyn FsDomain> {
    let procfs = Arc::new(ProcFs::new(CommonFsProviderImpl, "procfs"));
    Box::new(ProcFsDomain::new(
        procfs,
        "procfs".to_string(),
        Some(mount_func),
    ))
}

type ProcFsDirInodeImpl = DynFsDirInode<CommonFsProviderImpl, Mutex<()>>;

fn mount_func(root_dt: &Arc<dyn VfsDentry>) {
    let root_inode = root_dt.inode().unwrap();
    let root_inode = root_inode
        .downcast_arc::<ProcFsDirInodeImpl>()
        .map_err(|_| VfsError::Invalid)
        .unwrap();
    root_inode
        .add_file_manually("meminfo", Arc::new(MemInfo), "r--r--r--".into())
        .unwrap();
    root_inode
        .add_file_manually("interrupts", Arc::new(InterruptRecord), "r--r--r--".into())
        .unwrap();
    root_inode
        .add_file_manually("mounts", Arc::new(MountInfo), "r--r--r--".into())
        .unwrap();
    root_inode
        .add_file_manually(
            "filesystems",
            Arc::new(SystemSupportFS::new()),
            "r--r--r--".into(),
        )
        .unwrap();
    root_inode
        .add_dir_manually("self", "r-xr-xr-x".into())
        .unwrap();
}
