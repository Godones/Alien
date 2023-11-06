# 新的VFS实现和文件系统系统调用改进

## Why do this？

在之前的内核实现中，许多地方由于前期的设计不够良好而导致了冗余的实现，同时也出现了一些不必要的trick。这些冗余实现代码复杂，逻辑比较混乱，因此扩展性也很差，导致如果想要做改进很困难。而这部分的大头是在文件系统部分，文件系统相关的系统调用多达几十个。

虽然前面已经实现了一个看起来还可以的VFS框架以提高这部分的扩展性，但现在再来看之前的实现已经不够好了，因为当时我们参考并模仿了linux的VFS实现，虽然在C语言中那样的实现方式是有效的，但是将其照搬到`rust` 中似乎不太行，当然前期这些问题没有暴露出来，随着开发的推进，我们发现想要弥补之前没有实现的一些功能时，往VFS中添加代码变得愈加困难，这促使我们开始重构这部分的代码。

## How do this?

在新的实现中，我们尽量遵守`rust`的设计原则，同时也会阅读linux中的实现，以期达到一个更加优雅和更易扩展的实现。现在的VFS基本由`rust`的`trait`来进行定义。比如对于`Inode`的定义如下:

```rust
pub trait VfsInode: VfsFile {
    /// Get the super block of this dentry
    fn get_super_block(&self) -> VfsResult<Arc<dyn VfsSuperBlock>>;

    /// Get the permission of this inode
    fn node_perm(&self) -> VfsNodePerm;

    /// Create a new node with the given `path` in the directory
    fn create(
        &self,
        name: &str,
        ty: VfsNodeType,
        perm: VfsNodePerm,
        rdev: Option<u64>,
    ) -> VfsResult<Arc<dyn VfsInode>>;

    /// Create a new hard link to the src dentry
    fn link(&self, name: &str, src: Arc<dyn VfsInode>) -> VfsResult<Arc<dyn VfsInode>>;
    /// Remove hard link of file `name` from dir directory
    fn unlink(&self, name: &str) -> VfsResult<()>;
    /// Create a new symbolic link to the \[syn_name] file
    fn symlink(&self, name: &str, sy_name: &str) -> VfsResult<Arc<dyn VfsInode>>;
    fn lookup(&self, name: &str) -> VfsResult<Option<Arc<dyn VfsInode>>>;
    fn rmdir(&self, name: &str) -> VfsResult<()>;
    fn readlink(&self, buf: &mut [u8]) -> VfsResult<usize>;
    /// Set the attributes of the node.
    ///
    ///  This method is called by chmod(2) and related system calls.
    fn set_attr(&self, attr: InodeAttr) -> VfsResult<()>;
    /// Get the attributes of the node.
    ///
    /// This method is called by stat(2) and related system calls.
    fn get_attr(&self) -> VfsResult<VfsFileStat>;
    /// Called by the VFS to list all extended attributes for a given file.
    ///
    /// This method is called by the listxattr(2) system call.
    fn list_xattr(&self) -> VfsResult<Vec<String>>;
    fn inode_type(&self) -> VfsNodeType;
    fn truncate(&self, len: u64) -> VfsResult<()>;

    /// Rename the file `old_name` to `new_name` in the directory `new_parent`.
    fn rename_to(
        &self,
        old_name: &str,
        new_parent: Arc<dyn VfsInode>,
        new_name: &str,
        flag: VfsRenameFlag,
    ) -> VfsResult<()>;
    /// Update the access and modification times of the inode.
    ///
    /// This method is called by the utimensat(2) system call. The ctime will be updated automatically.
    ///
    /// The parameter `now` is used to update ctime.
    fn update_time(&self, time: VfsTime, now: VfsTimeSpec) -> VfsResult<()>;
}
```

当然还有其它一系列相关的数据结构定义，这些`trait`被小心地设计，以避免它的前身带来的问题。[vfscore](https://github.com/os-module/rvfs/tree/main/vfscore)是这个设计的核心，它包含了所有的定义和一些实用的工具来辅助使用者。

基于这个core，我们实现了几个基本的fs：

- fat-vfs
- ramfs
- devfs
- dynfs(as profs/sysfs ..)

除了`fat-vfs`之外，其它的都属于内存文件系统，因此在设计时我们尽量让这些文件系统共享一个数据结构，然后根据各个文件系统的差异调整具体的实现。

目前这些文件系统虽然覆盖了大部分接口，但仍有一部分接口是空的，而且缺乏测试，这是我们接下来需要做的。

## The kernel update

我们对内核中的跟文件相关的部分做了很大的修改，这涉及到了许多子系统：

- 设备文件
- pipe
- task
- syscall

因此这部分需要进行很多调整才能开始工作。这正在进行当中。在修改过程中我们发现新的实现大大简化了之前的实现，原本可能需要两百行的代码，现在只需要几十行就可以完成，这说明我们的重构卓有成效。

## The other implementation

在重构初期，我们广泛地调查了一些现有实现，这些实现都写的很好，但是也存在一些弊端，比如一些没有必要放在内核的实现被放在了内核当中，或者其实现依赖了内核数据结构。这也导致了我们无法直接使用他们的实现。在开发的过程中，我们从他们哪里吸收了宝贵的经验，也得到了很多有用的提示。

## The next

接下来的工作仍然会继续推进这部分的改进，直到我们对其进行重新测试并通过所有测例。目前我们的修改主要集中在内核部分，但也可能需要在VFS的框架中添加一部分代码。再接下来，我们会针对`mmap`以及一些其它的数据结构做改进。