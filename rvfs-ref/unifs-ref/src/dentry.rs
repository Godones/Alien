use alloc::{
    collections::BTreeMap,
    string::{String, ToString},
    sync::{Arc, Weak},
};

use vfscore::{
    dentry::VfsDentry, error::VfsError, fstype::VfsMountPoint, inode::VfsInode, utils::VfsNodeType,
    VfsResult,
};

use crate::VfsRawMutex;

pub struct UniFsDentry<R: VfsRawMutex> {
    inner: lock_api::Mutex<R, UniFsDentryInner<R>>,
}

struct UniFsDentryInner<R: VfsRawMutex> {
    parent: Weak<dyn VfsDentry>,
    inode: Arc<dyn VfsInode>,
    name: String,
    mnt: Option<VfsMountPoint>,
    children: Option<BTreeMap<String, Arc<UniFsDentry<R>>>>,
}

impl<R: VfsRawMutex + 'static> UniFsDentry<R> {
    /// Create the root dentry
    ///
    /// Only call once
    pub fn root(inode: Arc<dyn VfsInode>, parent: Weak<dyn VfsDentry>) -> Self {
        Self {
            inner: lock_api::Mutex::new(UniFsDentryInner {
                parent,
                inode,
                name: "/".to_string(),
                mnt: None,
                children: Some(BTreeMap::new()),
            }),
        }
    }

    pub fn new(inode: Arc<dyn VfsInode>, parent: Weak<dyn VfsDentry>, name: String) -> Self {
        Self {
            inner: lock_api::Mutex::new(UniFsDentryInner {
                parent,
                inode,
                name,
                mnt: None,
                children: None,
            }),
        }
    }
}

impl<R: VfsRawMutex + 'static> VfsDentry for UniFsDentry<R> {
    fn name(&self) -> String {
        self.inner.lock().name.clone()
    }

    fn to_mount_point(
        self: Arc<Self>,
        sub_fs_root: Arc<dyn VfsDentry>,
        mount_flag: u32,
    ) -> VfsResult<()> {
        let point = self as Arc<dyn VfsDentry>;
        let mnt = VfsMountPoint {
            root: sub_fs_root.clone(),
            mount_point: Arc::downgrade(&point),
            mnt_flags: mount_flag,
        };
        let point = point
            .downcast_arc::<UniFsDentry<R>>()
            .map_err(|_| VfsError::Invalid)?;
        let mut inner = point.inner.lock();
        inner.mnt = Some(mnt);
        Ok(())
    }

    fn inode(&self) -> VfsResult<Arc<dyn VfsInode>> {
        Ok(self.inner.lock().inode.clone())
    }

    fn mount_point(&self) -> Option<VfsMountPoint> {
        self.inner.lock().mnt.clone()
    }

    fn clear_mount_point(&self) {
        self.inner.lock().mnt = None;
    }

    fn find(&self, path: &str) -> Option<Arc<dyn VfsDentry>> {
        let inner = self.inner.lock();
        let inode_type = inner.inode.inode_type();
        match inode_type {
            VfsNodeType::Dir => inner
                .children
                .as_ref()
                .unwrap()
                .get(path)
                .map(|item| item.clone() as Arc<dyn VfsDentry>),
            _ => None,
        }
    }

    fn insert(
        self: Arc<Self>,
        name: &str,
        child: Arc<dyn VfsInode>,
    ) -> VfsResult<Arc<dyn VfsDentry>> {
        let inode_type = child.inode_type();
        let child = Arc::new(UniFsDentry {
            inner: lock_api::Mutex::new(UniFsDentryInner {
                parent: Arc::downgrade(&(self.clone() as Arc<dyn VfsDentry>)),
                inode: child,
                name: name.to_string(),
                mnt: None,
                children: match inode_type {
                    VfsNodeType::Dir => Some(BTreeMap::new()),
                    _ => None,
                },
            }),
        });
        self.inner
            .lock()
            .children
            .as_mut()
            .unwrap()
            .insert(name.to_string(), child.clone())
            .map_or(Ok(child), |_| Err(VfsError::EExist))
    }

    fn remove(&self, name: &str) -> Option<Arc<dyn VfsDentry>> {
        let mut inner = self.inner.lock();
        inner
            .children
            .as_mut()
            .unwrap()
            .remove(name)
            .map(|x| x as Arc<dyn VfsDentry>)
    }

    fn parent(&self) -> Option<Arc<dyn VfsDentry>> {
        self.inner.lock().parent.upgrade()
    }

    fn set_parent(&self, parent: &Arc<dyn VfsDentry>) {
        let mut inner = self.inner.lock();
        inner.parent = Arc::downgrade(parent);
    }
}
