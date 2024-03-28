#![no_std]
#![forbid(unsafe_code)]
extern crate alloc;
extern crate malloc;

use alloc::{string::ToString, sync::Arc};
use core::fmt::{Debug, Formatter, Write};

use basic::{println, write_console};
use fat_vfs::{FatFs, FatFsProvider};
use interface::{Basic, BlkDeviceDomain, DomainType, FsDomain};
use ksync::Mutex;
use log::debug;
use rref::RRef;
use vfscore::{
    dentry::VfsDentry,
    file::VfsFile,
    fstype::VfsFsType,
    inode::VfsInode,
    utils::{VfsFileStat, VfsNodePerm, VfsNodeType, VfsTimeSpec},
    VfsResult,
};

#[derive(Clone)]
pub struct FatFsDomain {
    root: Arc<dyn VfsDentry>,
}

impl Debug for FatFsDomain {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "FatFsDomain")
    }
}

impl FatFsDomain {
    pub fn new(root: Arc<dyn VfsDentry>) -> Self {
        Self { root }
    }
}

impl Basic for FatFsDomain {
    // fn drop_self(self: Box<Self>) {
    //     info!("Drop FatFsDomain");
    //     drop(self);
    // }
}

impl FsDomain for FatFsDomain {}

#[derive(Clone)]
struct ProviderImpl;
impl FatFsProvider for ProviderImpl {
    fn current_time(&self) -> VfsTimeSpec {
        VfsTimeSpec::new(0, 0)
    }
}

struct FakeInode {
    device: Mutex<Arc<dyn BlkDeviceDomain>>,
}

impl FakeInode {
    pub fn new(device: Arc<dyn BlkDeviceDomain>) -> Self {
        Self {
            device: Mutex::new(device),
        }
    }
}

impl VfsFile for FakeInode {
    fn read_at(&self, offset: u64, buf: &mut [u8]) -> VfsResult<usize> {
        let read_len = buf.len();
        let device = self.device.lock();
        let mut tmp_buf = RRef::new([0u8; 512]);

        let mut read_offset = offset;
        let mut count = 0;

        // 12 512
        while count < read_len {
            let block = read_offset / 512;
            let offset = read_offset % 512;
            let read_len = core::cmp::min(512 - offset as usize, read_len - count);
            tmp_buf = device.read_block(block as u32, tmp_buf).unwrap();
            buf[count..count + read_len]
                .copy_from_slice(&tmp_buf[offset as usize..offset as usize + read_len]);
            count += read_len;
            read_offset += read_len as u64;
        }
        Ok(count)
    }
    fn write_at(&self, offset: u64, buf: &[u8]) -> VfsResult<usize> {
        let write_len = buf.len();
        let device = self.device.lock();
        let mut tmp_buf = RRef::new([0u8; 512]);

        let mut write_offset = offset;
        let mut count = 0;

        // 12 512
        while count < write_len {
            let block = write_offset / 512;
            let offset = write_offset % 512;
            if offset != 0 {
                tmp_buf = device.read_block(block as u32, tmp_buf).unwrap();
            }
            let write_len = core::cmp::min(512 - offset as usize, write_len - count);
            tmp_buf[offset as usize..offset as usize + write_len]
                .copy_from_slice(&buf[count..count + write_len]);
            device.write_block(block as u32, &tmp_buf).unwrap();
            count += write_len;
            write_offset += write_len as u64;
        }
        Ok(count)
    }
    fn flush(&self) -> VfsResult<()> {
        self.device.lock().flush().unwrap();
        Ok(())
    }
    fn fsync(&self) -> VfsResult<()> {
        Ok(())
    }
}

impl VfsInode for FakeInode {
    fn node_perm(&self) -> VfsNodePerm {
        VfsNodePerm::from_bits_truncate(0x777)
    }
    fn get_attr(&self) -> VfsResult<VfsFileStat> {
        debug!("get_attr");
        Ok(VfsFileStat {
            st_dev: 0,
            st_ino: 0,
            st_mode: 0,
            st_nlink: 1,
            st_uid: 0,
            st_gid: 0,
            st_rdev: 0,
            __pad: 0,
            st_size: self.device.lock().get_capacity().unwrap(),
            st_blksize: 512,
            __pad2: 0,
            st_blocks: 0,
            st_atime: VfsTimeSpec::new(0, 0),
            st_mtime: VfsTimeSpec::new(0, 0),
            st_ctime: VfsTimeSpec::new(0, 0),
            unused: 0,
        })
    }
    fn inode_type(&self) -> VfsNodeType {
        VfsNodeType::BlockDevice
    }
}

pub fn main() -> Arc<dyn FsDomain> {
    let blk_device = basic::get_domain("blk").unwrap();
    let blk_device = match blk_device {
        DomainType::BlkDeviceDomain(blk) => blk,
        _ => panic!("devices domain not found"),
    };

    let fatfs = Arc::new(FatFs::<_, Mutex<()>>::new(ProviderImpl));
    let root = fatfs
        .clone()
        .mount(0, "/", Some(Arc::new(FakeInode::new(blk_device))), &[])
        .unwrap();
    println!("****Files In Root****");
    vfscore::path::print_fs_tree(&mut FakeOut, root.clone(), "".to_string(), true).unwrap();
    println!("List all file passed");
    Arc::new(FatFsDomain::new(root))
}

struct FakeOut;
impl Write for FakeOut {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        write_console(s);
        Ok(())
    }
}
