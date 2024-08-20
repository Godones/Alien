use std::sync::Arc;

use ramfs::{RamFs, RamFsProvider};
use spin::{Lazy, Mutex};
use vfscore::{
    dentry::VfsDentry,
    fstype::VfsFsType,
    path::VfsPath,
    utils::{VfsInodeMode, VfsNodeType, VfsTimeSpec},
    VfsResult,
};
use vfscore_ref as vfscore;

static FS: Lazy<Mutex<Arc<dyn VfsFsType>>> =
    Lazy::new(|| Mutex::new(Arc::new(RamFs::<_, Mutex<()>>::new(RamFsProviderImpl))));

#[derive(Clone)]
struct RamFsProviderImpl;
impl RamFsProvider for RamFsProviderImpl {
    fn current_time(&self) -> VfsTimeSpec {
        Default::default()
    }
}

fn make_ramfs() -> VfsResult<Arc<dyn VfsDentry>> {
    FS.lock().clone().mount(0, "/", None, &[])
}

#[test]
fn test_vfs_path() {
    let root = make_ramfs().unwrap();
    let path = VfsPath::new(root.clone(), root.clone());
    let rt = path.open(None);
    assert!(rt.is_ok());
    assert!(Arc::ptr_eq(&rt.unwrap(), &root));
    let path = path.join("d1").unwrap();
    let res = path.open(Some(
        VfsInodeMode::from_bits_truncate(0o777) | VfsInodeMode::FILE,
    ));
    assert!(res.is_ok());
    assert_eq!(path.extension(), None);
}

#[test]
fn test_dentry_path() {
    let root = make_ramfs().unwrap();
    let d1 = root
        .inode()
        .unwrap()
        .create("d1", VfsNodeType::Dir, "rwxrwxrwx".into(), None)
        .unwrap();
    let dd1 = d1
        .create("dd1", VfsNodeType::Dir, "rwxrwxrwx".into(), None)
        .unwrap();
    let d1_dt = root.i_insert("d1", d1.clone()).unwrap();
    let dd1_dt = d1_dt.i_insert("dd1", dd1.clone()).unwrap();
    // create a new ramfs
    let new_root = make_ramfs().unwrap();
    let new_d1 = new_root
        .inode()
        .unwrap()
        .create("d1", VfsNodeType::Dir, "rwxrwxrwx".into(), None)
        .unwrap();
    let new_dd1 = new_d1
        .create("dd1", VfsNodeType::Dir, "rwxrwxrwx".into(), None)
        .unwrap();
    let new_d1_dt = new_root.i_insert("d1", new_d1.clone()).unwrap();
    let new_dd1_dt = new_d1_dt.i_insert("dd1", new_dd1.clone()).unwrap();

    let path = VfsPath::new(root.clone(), dd1_dt.clone());
    // now new ramfs has been mounted to dd1_dt
    path.mount(new_root.clone(), 0).unwrap();

    let new_dd1_dt__ = path.join("d1/dd1").unwrap().open(None).unwrap();

    assert!(new_root.parent().is_some());
    assert!(Arc::ptr_eq(&dd1_dt, &new_root.parent().unwrap()));
    assert_eq!(new_dd1_dt__.path(), "/d1/dd1/d1/dd1");
    assert!(Arc::ptr_eq(&new_dd1_dt__, &new_dd1_dt));
    assert_eq!(new_dd1_dt__.name(), "dd1");

    let path = VfsPath::new(root.clone(), root);
    let new_dd1_dt___ = path.join("./d1/dd1/d1/dd1").unwrap().open(None).unwrap();
    assert!(Arc::ptr_eq(&new_dd1_dt___, &new_dd1_dt));

    let new_root_ = path.join("./d1/dd1").unwrap().open(None).unwrap();
    assert!(Arc::ptr_eq(&new_root_, &new_root));
    let path = new_root_.path();
    assert_eq!(path, "/d1/dd1");
}

#[test]
fn test_link() {}

#[test]
fn test_symlink() {}

#[test]
fn test_unlink() {}

#[test]
fn test_rename() {}
