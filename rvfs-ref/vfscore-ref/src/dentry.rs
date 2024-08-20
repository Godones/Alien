use alloc::{string::String, sync::Arc};

use downcast_rs::{impl_downcast, DowncastSync};
use log::warn;

use crate::{fstype::VfsMountPoint, inode::VfsInode, VfsResult};

pub trait VfsDentry: Send + Sync + DowncastSync {
    /// Return the name of this dentry
    fn name(&self) -> String;
    /// Make this dentry to  a mount point
    fn to_mount_point(
        self: Arc<Self>,
        sub_fs_root: Arc<dyn VfsDentry>,
        mount_flag: u32,
    ) -> VfsResult<()>;
    /// Get the inode of this dentry
    fn inode(&self) -> VfsResult<Arc<dyn VfsInode>>;
    /// Get the mount point of this dentry
    fn mount_point(&self) -> Option<VfsMountPoint>;
    /// Remove the mount point of this dentry
    fn clear_mount_point(&self);
    /// Whether this dentry is a mount point
    fn is_mount_point(&self) -> bool {
        self.mount_point().is_some()
    }
    /// Lookup a dentry in the directory
    ///
    /// The dentry should cache it's children to speed up the lookup
    fn find(&self, path: &str) -> Option<Arc<dyn VfsDentry>>;
    /// Insert a child to this dentry and return the dentry of the child
    fn insert(
        self: Arc<Self>,
        name: &str,
        child: Arc<dyn VfsInode>,
    ) -> VfsResult<Arc<dyn VfsDentry>>;
    /// Remove a child from this dentry and return the dentry of the child
    fn remove(&self, name: &str) -> Option<Arc<dyn VfsDentry>>;

    /// Get the parent of this dentry
    fn parent(&self) -> Option<Arc<dyn VfsDentry>>;

    /// Set the parent of this dentry
    ///
    /// This is useful when you want to move a dentry to another directory or
    /// mount this dentry to another directory
    fn set_parent(&self, parent: &Arc<dyn VfsDentry>);

    /// Get the path of this dentry
    fn path(&self) -> String {
        if let Some(p) = self.parent() {
            let path = if self.name() == "/" {
                String::from("")
            } else {
                String::from("/") + self.name().as_str()
            };
            let parent_name = p.name();
            if parent_name == "/" {
                if p.parent().is_some() {
                    // p is a mount point
                    p.parent().unwrap().path() + path.as_str()
                } else {
                    path
                }
            } else {
                // p is not root
                p.path() + path.as_str()
            }
        } else {
            warn!("dentry has no parent");
            String::from("/")
        }
    }
}

impl dyn VfsDentry {
    /// Insert a child to this dentry and return the dentry of the child
    ///
    /// It likes [`VfsDentry::insert`], but it will not take ownership of `self`
    pub fn i_insert(
        self: &Arc<Self>,
        name: &str,
        child: Arc<dyn VfsInode>,
    ) -> VfsResult<Arc<dyn VfsDentry>> {
        self.clone().insert(name, child)
    }
    /// Make this dentry to  a mount point
    ///
    /// It likes [`VfsDentry::to_mount_point`], but it will not take ownership of `self`
    pub fn i_to_mount_point(
        self: &Arc<Self>,
        sub_fs_root: Arc<dyn VfsDentry>,
        mount_flag: u32,
    ) -> VfsResult<()> {
        self.clone().to_mount_point(sub_fs_root, mount_flag)
    }
}

impl_downcast!(sync VfsDentry);
