use alloc::sync::Arc;

use bpf_basic::{
    linux_bpf::bpf_attr,
    map::{BpfMapMeta, UnifiedMap},
};
use constants::{io::SeekFrom, AlienResult, LinuxErrno};
use ksync::{Mutex, MutexGuard};
use vfs::kfile::File;
use vfscore::{dentry::VfsDentry, inode::VfsInode, utils::VfsFileStat};

use crate::{ebpf::transform::bpf_error_to_err, per_cpu::PerCpuImpl, task::current_task};

#[derive(Debug)]
pub struct BpfMap {
    unified_map: Mutex<UnifiedMap>,
}

impl BpfMap {
    pub fn new(unified_map: UnifiedMap) -> Self {
        BpfMap {
            unified_map: Mutex::new(unified_map),
        }
    }
    pub fn unified_map(&self) -> MutexGuard<UnifiedMap> {
        self.unified_map.lock()
    }
}

impl File for BpfMap {
    fn read(&self, _buf: &mut [u8]) -> AlienResult<usize> {
        panic!("BpfMap::read() should not be called");
    }

    fn write(&self, _buf: &[u8]) -> AlienResult<usize> {
        panic!("BpfMap::write() should not be called");
    }

    fn seek(&self, _pos: SeekFrom) -> AlienResult<u64> {
        panic!("BpfMap::seek() should not be called");
    }

    fn get_attr(&self) -> AlienResult<VfsFileStat> {
        let stat = VfsFileStat::default();
        Ok(stat)
    }

    fn dentry(&self) -> Arc<dyn VfsDentry> {
        panic!("BpfMap::dentry() should not be called");
    }

    fn inode(&self) -> Arc<dyn VfsInode> {
        panic!("BpfMap::inode() should not be called");
    }

    fn is_readable(&self) -> bool {
        false
    }

    fn is_writable(&self) -> bool {
        false
    }

    fn is_append(&self) -> bool {
        false
    }
}

pub fn bpf_map_create(attr: &bpf_attr) -> AlienResult<isize> {
    let map_meta = BpfMapMeta::try_from(attr).map_err(|_| LinuxErrno::EINVAL)?;
    let unified_map =
        bpf_basic::map::bpf_map_create::<PerCpuImpl>(map_meta).map_err(bpf_error_to_err)?;
    let file = Arc::new(BpfMap::new(unified_map));
    let task = current_task().unwrap();
    let fd = task.add_file(file).map_err(|_| LinuxErrno::EMFILE)?;
    Ok(fd as isize)
}
