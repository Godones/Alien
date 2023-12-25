#![no_std]
#![deny(unsafe_code)]
extern crate alloc;
extern crate malloc;

use alloc::boxed::Box;
use alloc::string::ToString;
use alloc::sync::Arc;
use core::fmt::Write;
use fat_vfs::{FatFs, FatFsProvider};
use interface::{Basic, BlkDevice, Fs};
use ksync::Mutex;
use libsyscall::println;
use log::info;
use rref::{RRef, RpcResult};
use vfscore::file::VfsFile;
use vfscore::fstype::VfsFsType;
use vfscore::inode::VfsInode;
use vfscore::utils::{VfsFileStat, VfsNodePerm, VfsNodeType, VfsTimeSpec};
use vfscore::VfsResult;

pub struct FatFsDomain {}

impl FatFsDomain {
    pub fn new() -> Self {
        Self {}
    }
}

impl Basic for FatFsDomain {
    fn drop_self(self: Box<Self>) {
        info!("Drop FatFsDomain");
        drop(self);
    }
}

impl Fs for FatFsDomain {
    fn ls(&self, _path: RRef<[u8; 512]>) -> RpcResult<RRef<[u8; 512]>> {
        unimplemented!()
    }
}

#[derive(Clone)]
struct ProviderImpl;
impl FatFsProvider for ProviderImpl {
    fn current_time(&self) -> VfsTimeSpec {
        VfsTimeSpec::new(0, 0)
    }
}

struct FakeInode {
    device: Mutex<Box<dyn BlkDevice>>,
}

impl FakeInode {
    pub fn new(device: Box<dyn BlkDevice>) -> Self {
        Self {
            device: Mutex::new(device),
        }
    }
}

impl VfsFile for FakeInode {
    fn read_at(&self, offset: u64, buf: &mut [u8]) -> VfsResult<usize> {
        let read_len = buf.len();
        let mut device = self.device.lock();
        let mut tmp_buf = RRef::new([0u8; 512]);

        let mut read_offset = offset;
        let mut count = 0;

        // 12 512
        while count < read_len {
            let block = read_offset / 512;
            let offset = read_offset % 512;
            let read_len = core::cmp::min(512 - offset as usize, read_len - count);
            tmp_buf = device.read(block as u32, tmp_buf).unwrap();
            buf[count..count + read_len]
                .copy_from_slice(&tmp_buf[offset as usize..offset as usize + read_len]);
            count += read_len;
            read_offset += read_len as u64;
        }
        Ok(count)
    }
    fn write_at(&self, offset: u64, buf: &[u8]) -> VfsResult<usize> {
        let write_len = buf.len();
        let mut device = self.device.lock();
        let mut tmp_buf = RRef::new([0u8; 512]);

        let mut write_offset = offset;
        let mut count = 0;

        // 12 512
        while count < write_len {
            let block = write_offset / 512;
            let offset = write_offset % 512;
            if offset != 0 {
                tmp_buf = device.read(block as u32, tmp_buf).unwrap();
            }
            let write_len = core::cmp::min(512 - offset as usize, write_len - count);
            tmp_buf[offset as usize..offset as usize + write_len]
                .copy_from_slice(&buf[count..count + write_len]);
            device.write(block as u32, &tmp_buf).unwrap();
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

pub fn main(blk_device: Box<dyn BlkDevice>) -> Box<dyn Fs> {
    let fatfs = Arc::new(FatFs::<_, Mutex<()>>::new(ProviderImpl));
    let root = fatfs
        .clone()
        .mount(0, "/", Some(Arc::new(FakeInode::new(blk_device))), &[])
        .unwrap();
    println!("****Files In Root****");
    vfscore::path::print_fs_tree(&mut FakeOut, root, "".to_string(), true).unwrap();
    Box::new(FatFsDomain::new())
}

struct FakeOut;
impl Write for FakeOut {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        libsyscall::write_console(s);
        Ok(())
    }
}
