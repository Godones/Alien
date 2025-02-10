use alloc::{string::String, sync::Arc, vec::Vec};

use downcast_rs::{impl_downcast, DowncastSync};
use shared_heap::DVec;

use crate::{
    error::VfsError,
    file::VfsFile,
    superblock::VfsSuperBlock,
    utils::{VfsFileStat, VfsNodePerm, VfsNodeType, VfsRenameFlag, VfsTime, VfsTimeSpec},
    VfsResult,
};

#[derive(Copy, Clone)]
pub struct InodeAttr {
    /// File mode.
    pub mode: u32,
    pub uid: u32,
    pub gid: u32,
    /// File size, in bytes.
    ///
    /// For truncate
    pub size: u64,
    pub atime: VfsTimeSpec,
    pub mtime: VfsTimeSpec,
    pub ctime: VfsTimeSpec,
}

pub trait VfsInode: DowncastSync + VfsFile {
    /// Get the super block of this dentry
    fn get_super_block(&self) -> VfsResult<Arc<dyn VfsSuperBlock>> {
        Err(VfsError::NoSys)
    }

    /// Get the permission of this inode
    fn node_perm(&self) -> VfsNodePerm {
        VfsNodePerm::empty()
    }

    /// Create a new node with the given `path` in the directory
    fn create(
        &self,
        _name: &str,
        _ty: VfsNodeType,
        _perm: VfsNodePerm,
        _rdev: Option<u64>,
    ) -> VfsResult<Arc<dyn VfsInode>> {
        Err(VfsError::NoSys)
    }

    /// Create a new hard link to the src dentry
    fn link(&self, _name: &str, _src: Arc<dyn VfsInode>) -> VfsResult<Arc<dyn VfsInode>> {
        Err(VfsError::NoSys)
    }
    /// Remove hard link of file `name` from dir directory
    fn unlink(&self, _name: &str) -> VfsResult<()> {
        Err(VfsError::NoSys)
    }
    /// Create a new symbolic link to the \[syn_name] file
    fn symlink(&self, _name: &str, _sy_name: &str) -> VfsResult<Arc<dyn VfsInode>> {
        Err(VfsError::NoSys)
    }
    fn lookup(&self, _name: &str) -> VfsResult<Arc<dyn VfsInode>> {
        Err(VfsError::NoSys)
    }
    fn rmdir(&self, _name: &str) -> VfsResult<()> {
        Err(VfsError::NoSys)
    }
    fn readlink(&self, _buf: DVec<u8>) -> VfsResult<(DVec<u8>, usize)> {
        Err(VfsError::NoSys)
    }
    /// Set the attributes of the node.
    ///
    ///  This method is called by chmod(2) and related system calls.
    fn set_attr(&self, _attr: InodeAttr) -> VfsResult<()> {
        Err(VfsError::NoSys)
    }
    /// Get the attributes of the node.
    ///
    /// This method is called by stat(2) and related system calls.
    fn get_attr(&self) -> VfsResult<VfsFileStat> {
        Err(VfsError::NoSys)
    }
    /// Called by the VFS to list all extended attributes for a given file.
    ///
    /// This method is called by the listxattr(2) system call.
    fn list_xattr(&self) -> VfsResult<Vec<String>> {
        Err(VfsError::NoSys)
    }
    fn inode_type(&self) -> VfsNodeType;
    fn truncate(&self, _len: u64) -> VfsResult<()> {
        Err(VfsError::NoSys)
    }

    /// Rename the file `old_name` to `new_name` in the directory `new_parent`.
    fn rename_to(
        &self,
        _old_name: &str,
        _new_parent: Arc<dyn VfsInode>,
        _new_name: &str,
        _flag: VfsRenameFlag,
    ) -> VfsResult<()> {
        Err(VfsError::NoSys)
    }
    /// Update the access and modification times of the inode.
    ///
    /// This method is called by the utimensat(2) system call. The ctime will be updated automatically.
    ///
    /// The parameter `now` is used to update ctime.
    fn update_time(&self, _time: VfsTime, _now: VfsTimeSpec) -> VfsResult<()> {
        Err(VfsError::NoSys)
    }
}

impl_downcast!(sync  VfsInode);

/// This macro is used to implement the default methods of `VfsInode` for inode which type is dir.
#[macro_export]
macro_rules! impl_dir_inode_default {
    () => {
        fn readlink(&self, _buf: DVec<u8>) -> VfsResult<(DVec<u8>, usize)> {
            Err(VfsError::IsDir)
        }
        fn truncate(&self, _len: u64) -> VfsResult<()> {
            Err(VfsError::IsDir)
        }
    };
}

/// This macro is used to implement the default methods of `VfsInode` for inode which type is symlink or other (not dir).
#[macro_export]
macro_rules! impl_common_inode_default {
    () => {
        fn create(
            &self,
            _name: &str,
            _ty: VfsNodeType,
            _perm: VfsNodePerm,
            _rdev: Option<u64>,
        ) -> VfsResult<Arc<dyn VfsInode>> {
            Err(VfsError::NoSys)
        }
        fn link(&self, _name: &str, _src: Arc<dyn VfsInode>) -> VfsResult<Arc<dyn VfsInode>> {
            Err(VfsError::NoSys)
        }
        fn unlink(&self, _name: &str) -> VfsResult<()> {
            Err(VfsError::NoSys)
        }
        fn symlink(&self, _name: &str, _target: &str) -> VfsResult<Arc<dyn VfsInode>> {
            Err(VfsError::NoSys)
        }
        fn lookup(&self, _name: &str) -> VfsResult<Arc<dyn VfsInode>> {
            Err(VfsError::NoSys)
        }
        fn rmdir(&self, _name: &str) -> VfsResult<()> {
            Err(VfsError::NoSys)
        }
        fn truncate(&self, _len: u64) -> VfsResult<()> {
            Err(VfsError::NoSys)
        }
        fn rename_to(
            &self,
            _old_name: &str,
            _new_parent: Arc<dyn VfsInode>,
            _new_name: &str,
            _flag: VfsRenameFlag,
        ) -> VfsResult<()> {
            Err(VfsError::NoSys)
        }
    };
}

/// This macro is used to implement the default methods of `VfsInode` for inode which type is file.
#[macro_export]
macro_rules! impl_file_inode_default {
    () => {
        fn create(
            &self,
            _name: &str,
            _ty: VfsNodeType,
            _perm: VfsNodePerm,
            _rdev: Option<u64>,
        ) -> VfsResult<Arc<dyn VfsInode>> {
            Err(VfsError::NoSys)
        }
        fn link(&self, _name: &str, _src: Arc<dyn VfsInode>) -> VfsResult<Arc<dyn VfsInode>> {
            Err(VfsError::NoSys)
        }
        fn unlink(&self, _name: &str) -> VfsResult<()> {
            Err(VfsError::NoSys)
        }
        fn symlink(&self, _name: &str, _target: &str) -> VfsResult<Arc<dyn VfsInode>> {
            Err(VfsError::NoSys)
        }
        fn lookup(&self, _name: &str) -> VfsResult<Arc<dyn VfsInode>> {
            Err(VfsError::NoSys)
        }
        fn rmdir(&self, _name: &str) -> VfsResult<()> {
            Err(VfsError::NoSys)
        }
        fn rename_to(
            &self,
            _old_name: &str,
            _new_parent: Arc<dyn VfsInode>,
            _new_name: &str,
            _flag: VfsRenameFlag,
        ) -> VfsResult<()> {
            Err(VfsError::NoSys)
        }
        fn readlink(&self, _buf: DVec<u8>) -> VfsResult<(DVec<u8>, usize)> {
            Err(VfsError::NoSys)
        }
    };
}
