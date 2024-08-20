use std::{error::Error, sync::Arc};

use fake_rref::fake_init_rref;
use log::{error, info};
use ramfs::{RamFs, RamFsProvider};
use ramfs_ref as ramfs;
use spin::mutex::Mutex;
use vfscore::{
    fstype::VfsFsType,
    path::DirIter,
    utils::{VfsNodePerm, VfsNodeType, VfsTimeSpec},
    RRefVec,
};

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    fake_init_rref();
    // create the fstype for ramfs
    let ramfs = Arc::new(RamFs::<_, Mutex<()>>::new(PageProviderImpl));
    // create a real ramfs
    // This function will return the root dentry of the ramfs
    let root = ramfs.clone().mount(0, "/", None, &[])?;
    // we can get the super block from the inode
    let sb = root.inode()?.get_super_block()?;
    // we can get the fstype from the super block
    let _fs = sb.fs_type();

    // we can get the inode from the dentry
    let root_inode = root.inode()?;
    // write dir will cause a error
    root_inode
        .write_at(0, &RRefVec::new(0, 10))
        .is_err()
        .then(|| error!("write to dir error"));

    // find in dentry cache first
    root.find("test")
        .is_none()
        .then(|| error!("find test file error"));

    // lookup in inode second
    root_inode
        .lookup("test")
        .is_err()
        .then(|| error!("lookup test file error"));

    // if we can't find in the dentry cache and inode, we will create a new inode
    let test_inode = root_inode.create(
        "test",
        VfsNodeType::File,
        VfsNodePerm::from_bits_truncate(0o777),
        None,
    )?;
    let _test_dentry = root.clone().insert("test", test_inode.clone())?;

    root.find("test")
        .is_some()
        .then(|| info!("find test file ok"));

    test_inode
        .write_at(0, &RRefVec::new(b'x', 10))
        .is_ok()
        .then(|| info!("write to file xxxxxxxxxx ok"));
    let buf = RRefVec::new(0, 10);
    let (buf, _r) = test_inode.read_at(0, buf).unwrap();
    println!(
        "read file ok, the content is {}",
        core::str::from_utf8(&buf).unwrap()
    );
    // create a mount point
    let mount_dir = root_inode.create(
        "mount_dir",
        VfsNodeType::Dir,
        VfsNodePerm::from_bits_truncate(0o777),
        None,
    )?;
    let mnt_dt = root.clone().insert("mount_dir", mount_dir)?;

    // create a new ramfs
    let new_ramfs_root = ramfs.clone().mount(0, "/", None, &[])?;
    let new_sb = new_ramfs_root.inode()?.get_super_block()?;
    // mount the ramfs to the mount_dir
    mnt_dt.clone().to_mount_point(new_ramfs_root.clone(), 0)?;
    new_ramfs_root.set_parent(&mnt_dt);
    mnt_dt
        .is_mount_point()
        .then(|| info!("create a mount point"));

    println!("root dir: ");
    // readdir
    root_inode.children().for_each(|x| println!("{}", x.name));
    // unmount the ramfs
    ramfs.kill_sb(sb)?;
    ramfs.kill_sb(new_sb)?;
    Ok(())
}

#[derive(Debug, Clone)]
struct PageProviderImpl;

impl RamFsProvider for PageProviderImpl {
    fn current_time(&self) -> VfsTimeSpec {
        VfsTimeSpec::new(0, 0)
    }
}
