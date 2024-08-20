//! Utilities for path manipulation.
//!
use alloc::{
    boxed::Box,
    format,
    string::{String, ToString},
    sync::Arc,
    vec::Vec,
};
use core::{
    error::Error,
    fmt::{write, Debug, Formatter, Write},
};

use log::{error, trace};
use rref::RRefVec;

use crate::{
    dentry::VfsDentry,
    error::VfsError,
    inode::VfsInode,
    utils::{VfsDirEntry, VfsInodeMode, VfsNodePerm, VfsNodeType, VfsRenameFlag},
    VfsResult,
};

/// The context of system call
///
/// In VfsPath, we need to check the permission of the user, so we need the context of system call
pub struct SysContext {
    pub pid: u64,
    pub uid: u64,
    pub gid: u64,
    pub cwd: Arc<dyn VfsDentry>,
    pub root: Arc<dyn VfsDentry>,
}

#[derive(Clone)]
pub struct VfsPath {
    /// The root of the file system
    root: Arc<dyn VfsDentry>,
    /// The directory to start searching from
    fs: Arc<dyn VfsDentry>,
    /// The path to search for
    path: String,
}

impl PartialEq for VfsPath {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path && Arc::ptr_eq(&self.fs, &other.fs)
    }
}

impl Eq for VfsPath {}

impl Debug for VfsPath {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("VfsPath({})", self.path))
    }
}

impl VfsPath {
    pub fn new(root: Arc<dyn VfsDentry>, start: Arc<dyn VfsDentry>) -> Self {
        Self {
            root,
            fs: start,
            path: "".to_string(),
        }
    }
    pub fn as_str(&self) -> &str {
        &self.path
    }
    /// Appends a path segment to this path, returning the result
    pub fn join(&self, path: impl AsRef<str>) -> VfsResult<Self> {
        Ok(VfsPath {
            root: self.root.clone(),
            path: self.path.clone() + "/" + path.as_ref(),
            fs: self.fs.clone(),
        })
    }
    pub fn root(&self) -> Self {
        VfsPath {
            root: self.root.clone(),
            path: "".to_string(),
            fs: self.fs.clone(),
        }
    }
    pub fn is_root(&self) -> bool {
        self.path.is_empty()
    }

    fn to_symlink(&self, symlink: Arc<dyn VfsDentry>) -> VfsResult<Arc<dyn VfsDentry>> {
        let inode = symlink.inode()?;
        let buf = RRefVec::new_uninit(255);
        let (buf, r) = inode.readlink(buf)?;
        if r > 255 {
            return Err(VfsError::Invalid);
        }
        let path = core::str::from_utf8(&buf.as_slice()[..r]).map_err(|_| VfsError::Invalid)?;
        if path.starts_with("/") {
            trace!("[to_symlink] absolute path: {}", path);
            // absolute path
            let new_path = Self::new(self.root.clone(), self.root.clone()).join(path)?;
            new_path.open(None)
        } else {
            trace!("[to_symlink] relative path: {}", path);
            // relative path
            let p = symlink.parent().unwrap();
            let new_path = Self::new(self.root.clone(), p).join(path)?;
            new_path.open(None)
        }
    }
    /// It same as [`open`], but it will follow the symlink according to the flag
    ///
    /// This function is just to avoid modifications to the code that uses the open function.
    #[cfg(feature = "linux_error")]
    pub fn open2(
        &self,
        mode: Option<VfsInodeMode>,
        flag: pconst::io::OpenFlags,
    ) -> VfsResult<Arc<dyn VfsDentry>> {
        self.__open(mode, !flag.contains(pconst::io::OpenFlags::O_NOFOLLOW))
    }
    // todo!(more flag support and permission check)
    /// Open or create a dentry
    ///
    /// If you need create the file if it doesn't exist, the parameter `mode` should be `Some(mode)`.
    ///
    /// It will follow the symlink by default, if you don't want to follow the symlink, you can use [`open2`]
    ///
    /// # Example
    /// ```compile_fail
    /// use vfscore::path::VfsPath;
    /// let path = VfsPath::new(root);
    /// let dentry = path.open(None);
    /// ```
    ///
    pub fn open(&self, mode: Option<VfsInodeMode>) -> VfsResult<Arc<dyn VfsDentry>> {
        self.__open(mode, true)
    }

    fn __open(&self, mode: Option<VfsInodeMode>, symlink: bool) -> VfsResult<Arc<dyn VfsDentry>> {
        let exist = self.exists();
        match exist {
            Ok(d) => {
                if symlink {
                    let inode = d.inode()?;
                    match inode.inode_type() {
                        VfsNodeType::SymLink => self.to_symlink(d),
                        _ => Ok(d),
                    }
                } else {
                    Ok(d)
                }
            }
            Err(e) => match e {
                VfsError::NoEntry if mode.is_some() => {
                    let mut ty = mode.unwrap() & VfsInodeMode::TYPE_MASK;
                    if ty.is_empty() {
                        ty = VfsInodeMode::FILE;
                    }
                    match ty {
                        VfsInodeMode::FILE => self.create_file(mode.unwrap().into()),
                        VfsInodeMode::DIR => self.create_dir(mode.unwrap().into()),
                        _ => Err(VfsError::Invalid),
                    }
                }
                _ => Err(e),
            },
        }
    }

    fn create_file(&self, perm: VfsNodePerm) -> VfsResult<Arc<dyn VfsDentry>> {
        self.create(VfsNodeType::File, perm, "create file")
    }

    fn create_dir(&self, perm: VfsNodePerm) -> VfsResult<Arc<dyn VfsDentry>> {
        self.create(VfsNodeType::Dir, perm, "create dir")
    }

    // todo! create flags
    fn create(
        &self,
        ty: VfsNodeType,
        perm: VfsNodePerm,
        action: &str,
    ) -> VfsResult<Arc<dyn VfsDentry>> {
        let parent = self.get_parent(action)?;
        // resolve mount point
        let dentry = real_dentry_down(parent);
        let file_name = self.path.rsplit('/').next();
        if file_name.is_none() {
            return Err(VfsError::Invalid);
        }
        let file_name = file_name.unwrap();
        // first, we find in dentry cache
        let file = dentry.find(file_name);
        if file.is_none() {
            // second, we find in inode cache or disk
            let file_inode = dentry.inode()?.lookup(file_name);
            match file_inode {
                Ok(x) => {
                    dentry.insert(file_name, x)?;
                    Err(VfsError::EExist)
                }
                Err(e) => {
                    if e == VfsError::NoEntry {
                        // if we can't find the inode, we create a new inode and insert it into dentry cache
                        let file_inode = dentry.inode()?.create(file_name, ty, perm, None)?;
                        let file = dentry.insert(file_name, file_inode)?;
                        Ok(file)
                    } else {
                        Err(e)
                    }
                }
            }
        } else {
            Err(VfsError::EExist)
        }
    }

    /// Checks whether parent is a directory
    fn get_parent(&self, action: &str) -> VfsResult<Arc<dyn VfsDentry>> {
        let parent = self.parent();
        let parent = parent.exists()?;
        if !parent.inode()?.inode_type().is_dir() {
            error!("Could not {}, parent path is not a directory", action);
            return Err(VfsError::NotDir);
        }
        Ok(parent)
    }
    pub fn parent(&self) -> Self {
        let index = self.path.rfind('/');
        index
            .map(|idx| VfsPath {
                root: self.root.clone(),
                path: self.path[..idx].to_string(),
                fs: self.fs.clone(),
            })
            .unwrap_or_else(|| self.root())
    }

    pub fn exists(&self) -> VfsResult<Arc<dyn VfsDentry>> {
        let mut parent = self.fs.clone();
        let mut path = self.path.as_str();
        loop {
            let (name, rest) = split_path(path);
            let parent_inode = parent.inode()?;
            if !parent_inode.inode_type().is_dir() && !name.is_empty() {
                return Err(VfsError::NotDir);
            }
            if name.is_empty() {
                break;
            }
            match name {
                "." => {}
                ".." => {
                    let real_parent = real_dentry_up(parent.clone());
                    if let Some(p) = real_parent.parent() {
                        parent = p;
                    }
                }
                _ => {
                    // resolve mount point
                    let dentry = real_dentry_down(parent.clone());
                    // first, we find in dentry cache
                    let sub_dentry = dentry.find(name);
                    if sub_dentry.is_none() {
                        // second, we find in inode cache or disk
                        let parent_inode = dentry.inode()?;
                        let sub_inode = parent_inode.lookup(name)?;
                        // if we find the inode, we insert it into dentry cache
                        let sub_dentry = dentry.insert(name, sub_inode)?;
                        parent = sub_dentry;
                    } else {
                        parent = sub_dentry.unwrap();
                    }
                }
            }
            if rest.is_none() {
                break;
            }
            path = rest.unwrap();
        }
        // resolve mount point
        let dentry = real_dentry_down(parent);
        Ok(dentry)
    }
    pub fn filename(&self) -> String {
        let index = self.path.rfind('/').map(|x| x + 1).unwrap_or(0);
        self.path[index..].to_string()
    }

    pub fn extension(&self) -> Option<String> {
        let filename = self.filename();
        let mut parts = filename.rsplitn(2, '.');
        let after = parts.next();
        let before = parts.next();
        match before {
            None | Some("") => None,
            _ => after.map(|x| x.to_string()),
        }
    }

    // todo! permission check
    pub fn mount(&self, root: Arc<dyn VfsDentry>, mount_flag: u32) -> VfsResult<()> {
        assert!(root.parent().is_none());
        let dir = self.open(None)?;
        let inode = dir.inode()?;
        if !inode
            .node_perm()
            .contains(VfsNodePerm::GROUP_EXEC | VfsNodePerm::OTHER_EXEC | VfsNodePerm::OWNER_EXEC)
        {
            return Err(VfsError::PermissionDenied);
        }
        if inode.inode_type() != VfsNodeType::Dir {
            return Err(VfsError::NotDir);
        }
        root.set_parent(&dir);
        dir.to_mount_point(root, mount_flag)?;
        Ok(())
    }

    // todo! check much things
    pub fn umount(&self) -> VfsResult<()> {
        let dir = self.open(None)?;
        if !dir.is_mount_point() {
            return Err(VfsError::Invalid);
        }
        let mnt = dir.mount_point().unwrap();
        dir.clear_mount_point();
        mnt.root.inode()?.get_super_block()?.sync_fs(false)?;
        Ok(())
    }

    pub fn truncate(&self, len: u64) -> VfsResult<()> {
        let dt = self.open(None).expect("truncate open failed");
        let inode = dt.inode()?;
        if inode.inode_type() == VfsNodeType::Dir {
            return Err(VfsError::IsDir);
        }
        // let fs = inode.get_super_block()?;
        // todo! access check
        if inode.node_perm().contains(VfsNodePerm::OTHER_WRITE) {
            return Err(VfsError::Access);
        }
        inode.truncate(len)?;
        Ok(())
    }

    pub fn symlink(&self, target: &str) -> VfsResult<()> {
        let this = self.open(None);
        match this {
            Ok(_) => Err(VfsError::EExist),
            Err(e) => match e {
                VfsError::NoEntry => {
                    let parent = self.get_parent("create symlink")?;
                    let parent_inode = parent.inode()?;
                    let name = self.filename();
                    assert!(!name.is_empty());
                    let inode = parent_inode.symlink(&name, target)?;
                    let _ = parent.insert(&name, inode)?;
                    Ok(())
                }
                _ => Err(e),
            },
        }
    }
    pub fn link(&self, old_dt: Arc<dyn VfsDentry>) -> VfsResult<()> {
        if old_dt.inode()?.inode_type() == VfsNodeType::Dir {
            return Err(VfsError::PermissionDenied);
        }
        let this = self.open(None);
        match this {
            Ok(_) => Err(VfsError::EExist),
            Err(e) => match e {
                VfsError::NoEntry => {
                    let parent = self.get_parent("create hard link")?;
                    let parent_inode = parent.inode()?;

                    let old_fs = old_dt.inode()?.get_super_block()?;
                    let this_fs = parent_inode.get_super_block()?;
                    if !Arc::ptr_eq(&old_fs, &this_fs) {
                        return Err(VfsError::Invalid);
                    }
                    // todo! access check
                    let name = self.filename();
                    assert!(!name.is_empty());
                    let inode = parent_inode.link(&name, old_dt.inode()?)?;
                    let _ = parent.insert(&name, inode)?;
                    Ok(())
                }
                _ => Err(e),
            },
        }
    }

    pub fn rmdir(&self) -> VfsResult<()> {
        let dt = self.open(None)?;
        let inode = dt.inode()?;
        if inode.inode_type() != VfsNodeType::Dir {
            return Err(VfsError::NotDir);
        }
        let parent = self.get_parent("rmdir")?;
        let parent_inode = parent.inode()?;
        let perm = parent_inode.node_perm();

        if !perm.contains(VfsNodePerm::OTHER_WRITE)
            && !perm.contains(VfsNodePerm::GROUP_WRITE)
            && !perm.contains(VfsNodePerm::OWNER_WRITE)
        {
            return Err(VfsError::Access);
        }
        let name = self.filename();
        assert!(!name.is_empty());
        // todo! access check
        parent_inode.rmdir(&name)?;
        // remove the dentry from cache
        parent.remove(&name);
        Ok(())
    }

    pub fn unlink(&self) -> VfsResult<()> {
        let dt = self.open(None)?;
        let inode = dt.inode()?;
        if inode.inode_type() == VfsNodeType::Dir {
            return Err(VfsError::IsDir);
        }
        let parent = self.get_parent("unlink")?;
        let parent_inode = parent.inode()?;

        let perm = parent_inode.node_perm();

        if !perm.contains(VfsNodePerm::OTHER_WRITE)
            && !perm.contains(VfsNodePerm::GROUP_WRITE)
            && !perm.contains(VfsNodePerm::OWNER_WRITE)
        {
            return Err(VfsError::Access);
        }
        let name = self.filename();
        assert!(!name.is_empty());
        // todo! access check
        parent_inode.unlink(&name)?;

        // remove the dentry from cache
        parent.remove(&name);
        Ok(())
    }

    pub fn rename_to(
        &self,
        context: SysContext,
        new_vfs_path: VfsPath,
        flag: VfsRenameFlag,
    ) -> VfsResult<()> {
        let old_dt = self.open(None)?;
        checkout_busy(&old_dt, &context)?;
        let new_dt = new_vfs_path.open(None);
        if new_dt.is_err() {
            let err = new_dt.err().unwrap();
            if err != VfsError::NoEntry {
                return Err(err);
            }
            // new path not exist
            if flag.contains(VfsRenameFlag::RENAME_EXCHANGE) {
                return Err(VfsError::NoEntry);
            }
            let new_parent = new_vfs_path
                .get_parent("rename")
                .expect("get parent of new path failed, this should not happen");
            let old_parent = self
                .get_parent("rename")
                .expect("get parent of old path failed, this should not happen");

            check_same_fs(&new_parent, &old_parent)?;
            checkout_write_perm(&new_parent)?;
            checkout_write_perm(&old_parent)?;

            let old_parent_inode = old_parent.inode()?;

            old_parent_inode.rename_to(
                self.filename().as_str(),
                new_parent.inode()?,
                new_vfs_path.filename().as_str(),
                flag,
            )?;

            // remove the dentry from cache
            old_parent.remove(self.filename().as_str());
            // insert the dentry into cache
            new_parent.insert(new_vfs_path.filename().as_str(), old_dt.inode()?)?;
        } else {
            let new_dt = new_dt.unwrap();
            checkout_busy(&new_dt, &context)?;
            if flag.contains(VfsRenameFlag::RENAME_NOREPLACE) {
                return Err(VfsError::EExist);
            }
            // EISDIR newpath is an existing directory, but oldpath is not a directory.
            if new_dt.inode()?.inode_type() == VfsNodeType::Dir
                && old_dt.inode()?.inode_type() != VfsNodeType::Dir
            {
                return Err(VfsError::IsDir);
            }
            let new_parent = new_vfs_path
                .get_parent("rename")
                .expect("get parent of new path failed, this should not happen");
            let old_parent = self
                .get_parent("rename")
                .expect("get parent of old path failed, this should not happen");

            check_same_fs(&new_parent, &old_parent)?;
            checkout_write_perm(&new_parent)?;
            checkout_write_perm(&old_parent)?;
            let old_parent_inode = old_parent.inode()?;
            let new_parent_inode = new_parent.inode()?;
            old_parent_inode.rename_to(
                self.filename().as_str(),
                new_parent_inode,
                new_vfs_path.filename().as_str(),
                flag,
            )?;
            // remove the dentry from cache
            old_parent.remove(self.filename().as_str());
            new_parent.remove(new_vfs_path.filename().as_str());
            // insert the dentry into cache
            new_parent.insert(new_vfs_path.filename().as_str(), old_dt.inode()?)?;
            if flag.contains(VfsRenameFlag::RENAME_EXCHANGE) {
                // insert the dentry into cache
                old_parent.insert(self.filename().as_str(), new_dt.inode()?)?;
            } // 只有在交换的时候才需要插入,否则新的文件已经被覆盖掉了
        }
        Ok(())
    }
    pub fn set_xattr(&self, _key: &str, _value: &[u8]) -> VfsResult<()> {
        unimplemented!();
    }
    pub fn get_xattr(&self, _key: &str) -> VfsResult<Vec<u8>> {
        unimplemented!();
    }
}

/// Check whether the dentry has write permission
fn checkout_write_perm(dentry: &Arc<dyn VfsDentry>) -> VfsResult<()> {
    let perm = dentry.inode()?.node_perm();
    if !perm.contains(VfsNodePerm::OTHER_WRITE)
        && !perm.contains(VfsNodePerm::GROUP_WRITE)
        && !perm.contains(VfsNodePerm::OWNER_WRITE)
    {
        Err(VfsError::Access)
    } else {
        Ok(())
    }
}

/// Check whether the dentry is busy which means it is cwd or root
fn checkout_busy(dentry: &Arc<dyn VfsDentry>, context: &SysContext) -> VfsResult<()> {
    let b = dentry.inode()?.inode_type() == VfsNodeType::Dir && Arc::ptr_eq(dentry, &context.cwd)
        || Arc::ptr_eq(dentry, &context.root);
    if b {
        return Err(VfsError::EBUSY);
    }
    Ok(())
}
/// Check whether the two dentry is in the same fs
fn check_same_fs(dentry1: &Arc<dyn VfsDentry>, dentry2: &Arc<dyn VfsDentry>) -> VfsResult<()> {
    let fs1 = dentry1.inode()?.get_super_block()?;
    let fs2 = dentry2.inode()?.get_super_block()?;
    if !Arc::ptr_eq(&fs1, &fs2) {
        return Err(VfsError::Invalid);
    }
    Ok(())
}

fn real_dentry_down(dentry: Arc<dyn VfsDentry>) -> Arc<dyn VfsDentry> {
    if dentry.is_mount_point() {
        let mnt = dentry.mount_point().unwrap();
        real_dentry_down(mnt.root)
    } else {
        dentry
    }
}

/// "/bin/x/"
fn real_dentry_up(dentry: Arc<dyn VfsDentry>) -> Arc<dyn VfsDentry> {
    if dentry.name() == "/" {
        if let Some(parent) = dentry.parent() {
            real_dentry_up(parent)
        } else {
            dentry
        }
    } else {
        dentry
    }
}

fn split_path(path: &str) -> (&str, Option<&str>) {
    let trimmed_path = path.trim_start_matches('/');
    trimmed_path.find('/').map_or((trimmed_path, None), |n| {
        (&trimmed_path[..n], Some(&trimmed_path[n + 1..]))
    })
}

#[allow(unused)]
fn canonicalize(path: &str) -> String {
    let mut buf = String::new();
    let is_absolute = path.starts_with('/');
    for part in path.split('/') {
        match part {
            "" | "." => continue,
            ".." => {
                while !buf.is_empty() {
                    if buf == "/" {
                        break;
                    }
                    let c = buf.pop().unwrap();
                    if c == '/' {
                        break;
                    }
                }
            }
            _ => {
                if buf.is_empty() {
                    if is_absolute {
                        buf.push('/');
                    }
                } else if &buf[buf.len() - 1..] != "/" {
                    buf.push('/');
                }
                buf.push_str(part);
            }
        }
    }
    if is_absolute && buf.is_empty() {
        buf.push('/');
    }
    buf
}

#[test]
fn test_canonicalize() {
    // assert_eq!(canonicalize("."), "");
    // assert_eq!(canonicalize(".."), "..");
    assert_eq!(canonicalize(""), "");
    assert_eq!(canonicalize("///"), "/");
    assert_eq!(canonicalize("//a//.//b///c//"), "/a/b/c");
    assert_eq!(canonicalize("/a/../"), "/");
    assert_eq!(canonicalize("/a/../..///"), "/");
    assert_eq!(canonicalize("a/../"), "");
    assert_eq!(canonicalize("a/..//.."), "");
    assert_eq!(canonicalize("././a"), "a");
    assert_eq!(canonicalize(".././a"), "a");
    assert_eq!(canonicalize("/././a"), "/a");
    assert_eq!(canonicalize("/abc/../abc"), "/abc");
    assert_eq!(canonicalize("/test"), "/test");
    assert_eq!(canonicalize("/test/"), "/test");
    assert_eq!(canonicalize("test/"), "test");
    assert_eq!(canonicalize("test"), "test");
    assert_eq!(canonicalize("/test//"), "/test");
    assert_eq!(canonicalize("/test/foo"), "/test/foo");
    assert_eq!(canonicalize("/test/foo/"), "/test/foo");
    assert_eq!(canonicalize("/test/foo/bar"), "/test/foo/bar");
    assert_eq!(canonicalize("/test/foo/bar//"), "/test/foo/bar");
    assert_eq!(canonicalize("/test//foo/bar//"), "/test/foo/bar");
    assert_eq!(canonicalize("/test//./foo/bar//"), "/test/foo/bar");
    assert_eq!(canonicalize("/test//./.foo/bar//"), "/test/.foo/bar");
    assert_eq!(canonicalize("/test//./..foo/bar//"), "/test/..foo/bar");
    assert_eq!(canonicalize("/test//./../foo/bar//"), "/foo/bar");
    assert_eq!(canonicalize("/test/../foo"), "/foo");
    assert_eq!(canonicalize("/test/bar/../foo"), "/test/foo");
    assert_eq!(canonicalize("../foo"), "foo");
    assert_eq!(canonicalize("../foo/"), "foo");
    assert_eq!(canonicalize("/../foo"), "/foo");
    assert_eq!(canonicalize("/../foo/"), "/foo");
    assert_eq!(canonicalize("/../../foo"), "/foo");
    assert_eq!(canonicalize("/bleh/../../foo"), "/foo");
    assert_eq!(canonicalize("/bleh/bar/../../foo"), "/foo");
    assert_eq!(canonicalize("/bleh/bar/../../foo/.."), "/");
    assert_eq!(canonicalize("/bleh/bar/../../foo/../meh"), "/meh");
}

const B: u64 = 1024;
const KB: u64 = 1024 * 1024;
const MB: u64 = 1024 * 1024 * 1024;
const GB: u64 = 1024 * 1024 * 1024 * 1024;

fn size_to_str(size: u64) -> String {
    match size {
        0..B => format!("{}B", size),
        B..KB => format!("{}KB", size / B),
        KB..MB => format!("{}MB", size / KB),
        MB..GB => format!("{:.2}GB", size / MB),
        _ => format!("{:.2}TB", size / GB),
    }
}

pub fn print_fs_tree(
    output: &mut dyn Write,
    root: Arc<dyn VfsDentry>,
    prefix: String,
    recursive: bool,
) -> Result<(), Box<dyn Error>> {
    let mut children = root.inode()?.children();
    let mut child = children.next();
    while let Some(c) = child {
        let name = c.name;
        let inode_type = c.ty;
        let inode = root.inode()?.lookup(&name)?;
        let stat = inode.get_attr()?;
        let perm = VfsNodePerm::from_bits_truncate(stat.st_mode as u16);
        let rwx_buf = perm.rwx_buf();
        let rwx = core::str::from_utf8(&rwx_buf)?;

        let buf = RRefVec::new_uninit(32);
        let option = if inode_type == VfsNodeType::SymLink {
            let (buf, r) = inode.readlink(buf)?;
            let content = core::str::from_utf8(&buf.as_slice()[..r])?;
            "-> ".to_string() + content
        } else {
            "".to_string()
        };
        write(
            output,
            format_args!(
                "{}{}{} {:>8} {} {}\n",
                prefix,
                inode_type.as_char(),
                rwx,
                size_to_str(stat.st_size),
                name,
                option
            ),
        )
        .unwrap();

        if inode_type == VfsNodeType::Dir && name != "." && name != ".." && recursive {
            let d = root.find(&name);
            let sub_dt = if let Some(d) = d {
                d
            } else {
                let d = root.inode()?.lookup(&name)?;
                root.i_insert(&name, d)?
            };
            let sub_dt = real_dentry_down(sub_dt);
            print_fs_tree(output, sub_dt, prefix.clone() + "  ", recursive)?;
        }
        child = children.next();
    }
    Ok(())
}

pub trait DirIter {
    fn children(&self) -> Box<dyn Iterator<Item = VfsDirEntry>>;
}

struct DirIterImpl {
    inode: Arc<dyn VfsInode>,
    index: usize,
}
impl Iterator for DirIterImpl {
    type Item = VfsDirEntry;
    fn next(&mut self) -> Option<Self::Item> {
        let x = self.inode.readdir(self.index).unwrap();
        if let Some(x) = x {
            self.index += 1;
            Some(x)
        } else {
            None
        }
    }
}

impl DirIter for Arc<dyn VfsInode> {
    fn children(&self) -> Box<dyn Iterator<Item = VfsDirEntry>> {
        Box::new(DirIterImpl {
            inode: self.clone(),
            index: 0,
        })
    }
}

#[cfg(test)]
mod tests {
    use alloc::{string::String, sync::Arc};

    use crate::{
        dentry::VfsDentry,
        fstype::VfsMountPoint,
        inode::VfsInode,
        path::{split_path, VfsPath},
        VfsResult,
    };

    struct FakeDentry;
    impl VfsDentry for FakeDentry {
        fn name(&self) -> String {
            todo!()
        }

        fn to_mount_point(
            self: Arc<Self>,
            _sub_fs_root: Arc<dyn VfsDentry>,
            _mount_flag: u32,
        ) -> VfsResult<()> {
            todo!()
        }

        fn inode(&self) -> VfsResult<Arc<dyn VfsInode>> {
            todo!()
        }

        fn mount_point(&self) -> Option<VfsMountPoint> {
            todo!()
        }

        fn clear_mount_point(&self) {
            todo!()
        }

        fn find(&self, _path: &str) -> Option<Arc<dyn VfsDentry>> {
            todo!()
        }

        fn insert(
            self: Arc<Self>,
            _name: &str,
            _child: Arc<dyn VfsInode>,
        ) -> VfsResult<Arc<dyn VfsDentry>> {
            todo!()
        }

        fn remove(&self, _name: &str) -> Option<Arc<dyn VfsDentry>> {
            todo!()
        }

        fn parent(&self) -> Option<Arc<dyn VfsDentry>> {
            todo!()
        }

        fn set_parent(&self, _parent: &Arc<dyn VfsDentry>) {
            todo!()
        }
    }

    #[test]
    fn test_split_path() {
        assert_eq!(split_path("/foo/bar.txt"), ("foo", Some("bar.txt")));
        assert_eq!(split_path("/foo/bar"), ("foo", Some("bar")));
        assert_eq!(split_path("/foo"), ("foo", None));
        assert_eq!(split_path("/"), ("", None));
        assert_eq!(split_path(""), ("", None));
    }

    #[test]
    fn test_join() {
        let path = VfsPath::new(Arc::new(FakeDentry), Arc::new(FakeDentry));

        assert_eq!(path.join("foo.txt").unwrap().as_str(), "/foo.txt");
        assert_eq!(path.join("foo/bar.txt").unwrap().as_str(), "/foo/bar.txt");

        let foo = path.join("foo").unwrap();

        assert_eq!(
            path.join("foo/bar.txt").unwrap(),
            foo.join("bar.txt").unwrap()
        );
        // assert_eq!(path, path.join(".").unwrap());
        // assert_eq!(path, path.join("./").unwrap());
        assert_eq!(path.join("..").unwrap().as_str(), "/..");
    }

    #[test]
    fn test_path_filename() {
        let path = VfsPath::new(Arc::new(FakeDentry), Arc::new(FakeDentry));
        assert_eq!(path.join("foo.txt").unwrap().filename(), "foo.txt");
        assert_eq!(path.join("foo/bar.txt").unwrap().filename(), "bar.txt");
        assert_eq!(path.join("/foo").unwrap().filename(), "foo");
    }
}
