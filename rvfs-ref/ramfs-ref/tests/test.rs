use std::sync::{Arc, Once};

use fake_rref::fake_init_rref;
use ramfs::{RamFs, RamFsProvider};
use ramfs_ref as ramfs;
use spin::{mutex::Mutex, Lazy};
use vfscore::{
    dentry::VfsDentry,
    fstype::VfsFsType,
    path::DirIter,
    utils::{VfsNodeType, VfsTimeSpec},
    RRefVec, VfsResult,
};

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

fn init_rref() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| fake_init_rref());
}

#[test]
fn test_link() {
    init_rref();
    let root = make_ramfs().unwrap();
    let f1 = root
        .inode()
        .unwrap()
        .create("f1", VfsNodeType::File, "rwxrwxrwx".into(), None)
        .unwrap();
    let f1link = root.inode().unwrap().link("f1link", f1.clone()).unwrap();
    assert!(Arc::ptr_eq(&f1, &f1link));
    let num = root.inode().unwrap().children().fold(0, |acc, _| acc + 1);
    assert_eq!(num, 2);
    let attr = f1link.get_attr().unwrap();
    assert_eq!(attr.st_nlink, 2);

    let n_attr = f1.get_attr().unwrap();
    assert_eq!(attr, n_attr);

    root.inode().unwrap().unlink("f1link").unwrap();

    let num = root.inode().unwrap().children().fold(0, |acc, _| acc + 1);
    assert_eq!(num, 1);
    let attr = f1link.get_attr().unwrap();
    assert_eq!(attr.st_nlink, 1);
}

#[test]
fn test_symlink() {
    init_rref();
    let root = make_ramfs().unwrap();
    let f1 = root
        .inode()
        .unwrap()
        .create("f1", VfsNodeType::File, "rwxrwxrwx".into(), None)
        .unwrap();
    let f1_sym = root.inode().unwrap().symlink("f1_sym", "f1").unwrap();
    let num = root.inode().unwrap().children().fold(0, |acc, _| acc + 1);
    assert_eq!(num, 2);
    let attr = f1_sym.get_attr().unwrap();
    assert_eq!(attr.st_nlink, 1);
    assert_eq!(attr.st_size, 2);
    let n_attr = f1.get_attr().unwrap();
    assert_eq!(n_attr.st_nlink, 1);
    root.inode().unwrap().unlink("f1_sym").unwrap();
    let num = root.inode().unwrap().children().fold(0, |acc, _| acc + 1);
    assert_eq!(num, 1);

    let buf = RRefVec::new(0, 2);
    let (buf, _r) = f1_sym.readlink(buf).unwrap();
    assert_eq!(buf.as_slice(), b"f1");
}

#[test]
fn test_unlink() {}

#[test]
fn test_rename() {}
