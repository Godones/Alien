use alloc::{
    string::{String, ToString},
    sync::Weak,
    vec::Vec,
};

use vfscore::{
    inode::InodeAttr,
    utils::{VfsDirEntry, VfsFileStat, VfsNodePerm, VfsNodeType, VfsRenameFlag, VfsTime},
};

use crate::{UniFsSuperBlock, *};

pub struct UniFsInodeSame<T: Send + Sync, R: VfsRawMutex> {
    pub sb: Weak<UniFsSuperBlock<R>>,
    pub inode_number: u64,
    pub provider: T,
    pub inner: lock_api::Mutex<R, UniFsInodeAttr>,
}

pub struct UniFsInodeAttr {
    pub link_count: u32,
    pub atime: VfsTimeSpec,
    pub mtime: VfsTimeSpec,
    pub ctime: VfsTimeSpec,
    pub perm: VfsNodePerm,
}

pub fn basic_file_stat<T: Send + Sync, R: VfsRawMutex>(
    basic: &UniFsInodeSame<T, R>,
) -> VfsFileStat {
    let inner = basic.inner.lock();
    VfsFileStat {
        st_dev: 0,
        st_ino: basic.inode_number,
        st_mode: inner.perm.bits() as u32,
        st_nlink: inner.link_count,
        st_uid: 0,
        st_gid: 0,
        st_rdev: 0,
        __pad: 0,
        st_size: 4096,
        st_blksize: 4096,
        __pad2: 0,
        st_blocks: 0,
        st_atime: inner.atime,
        st_mtime: inner.mtime,
        st_ctime: inner.ctime,
        unused: 0,
    }
}

pub struct UniFsDirInode<T: Send + Sync, R: VfsRawMutex> {
    pub basic: UniFsInodeSame<T, R>,
    pub children: lock_api::Mutex<R, Vec<(String, u64)>>,
}

impl<T: Send + Sync + 'static, R: VfsRawMutex + 'static> UniFsDirInode<T, R> {
    pub fn readdir(&self, start_index: usize) -> VfsResult<Option<VfsDirEntry>> {
        let sb = self.basic.sb.upgrade().unwrap();
        let children = self.children.lock();
        let res = children
            .iter()
            .nth(start_index)
            .map(|(name, inode_number)| {
                let inode = sb
                    .get_inode(*inode_number)
                    .unwrap_or_else(|| panic!("inode {} not found in superblock", inode_number,));
                VfsDirEntry {
                    ino: *inode_number,
                    ty: inode.inode_type(),
                    name: name.clone(),
                }
            });
        Ok(res)
    }

    pub fn get_super_block(&self) -> VfsResult<Arc<dyn VfsSuperBlock>> {
        let res = self.basic.sb.upgrade().ok_or(VfsError::Invalid);
        res.map(|sb| sb as Arc<dyn VfsSuperBlock>)
    }

    pub fn lookup(&self, name: &str) -> VfsResult<Arc<dyn VfsInode>> {
        let sb = self.basic.sb.upgrade().unwrap();
        let res = self
            .children
            .lock()
            .iter()
            .find(|(item_name, _item)| item_name.as_str() == name)
            .map(|(_, inode_number)| sb.get_inode(*inode_number).unwrap());
        if let Some(res) = res {
            Ok(res)
        } else {
            Err(VfsError::NoEntry)
        }
    }

    #[inline]
    pub fn set_attr(&self, _attr: InodeAttr) -> VfsResult<()> {
        Ok(())
    }
    #[inline]
    pub fn get_attr(&self) -> VfsResult<VfsFileStat> {
        Ok(basic_file_stat(&self.basic))
    }
    #[inline]
    pub fn inode_type(&self) -> VfsNodeType {
        VfsNodeType::Dir
    }

    pub fn node_perm(&self) -> VfsNodePerm {
        self.basic.inner.lock().perm
    }

    pub fn update_time(&self, time: VfsTime, now: VfsTimeSpec) -> VfsResult<()> {
        let mut inner = self.basic.inner.lock();
        match time {
            VfsTime::AccessTime(t) => inner.atime = t,
            VfsTime::ModifiedTime(t) => inner.mtime = t,
        }
        inner.ctime = now;
        Ok(())
    }
    pub fn rename_to(
        &self,
        old_name: &str,
        new_parent: &UniFsDirInode<T, R>,
        new_name: &str,
        flag: VfsRenameFlag,
    ) -> VfsResult<()> {
        let old_inode_index = self
            .children
            .lock()
            .iter()
            .position(|(n, _)| n == old_name)
            .ok_or(VfsError::NoEntry)?;
        let new_inode = new_parent;
        let sb = self
            .get_super_block()?
            .downcast_arc::<UniFsSuperBlock<R>>()
            .map_err(|_| VfsError::Invalid)?;
        if flag.contains(VfsRenameFlag::RENAME_EXCHANGE) {
            // the old_name and new_name must exist
            let new_inode_index = new_inode
                .children
                .lock()
                .iter()
                .position(|(n, _)| n == new_name)
                .ok_or(VfsError::NoEntry)?;

            let (_, old_inode_number) = self.children.lock().remove(old_inode_index);
            let (_, new_inode_number) = new_inode.children.lock().remove(new_inode_index);

            self.children
                .lock()
                .push((new_name.to_string(), new_inode_number));
            new_inode
                .children
                .lock()
                .push((old_name.to_string(), old_inode_number));
        } else {
            let (_, old_inode_number) = self.children.lock().remove(old_inode_index);
            // the new_name may exist or not
            // we only need to delete it if it exists
            let new_inode_index = new_inode
                .children
                .lock()
                .iter()
                .position(|(n, _)| n == new_name);
            if let Some(new_inode_index) = new_inode_index {
                let (_, new_inode_number) = new_inode.children.lock().remove(new_inode_index);
                sb.remove_inode(new_inode_number);
            }
            self.children
                .lock()
                .push((new_name.to_string(), old_inode_number));
        }
        Ok(())
    }
}
