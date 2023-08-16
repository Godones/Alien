//! 共享内存是一种最为高效的进程间通信方式。因为进程可以直接读写内存，不需要任何数据的拷贝。
//! 为了在多个进程间交换信息，内核专门留出了一块内存区。这段内存区可以由需要访问的进程将其映射到自己的私有地址空间。
//! 因此，进程就可以直接读写这一内存区而不需要进行数据的拷贝，从而大大提高了效率。
//! 
//! 共享内存并未提供同步机制，也就是说，在第一个进程结束对共享内存的写操作之前，并无自动机制可以阻止第二个进程开始对它进行读取。
//! 所以我们通常需要用其他的机制来同步对共享内存的访问，如互斥锁和信号量等。
//! 

use alloc::collections::btree_map::BTreeMap;
use alloc::vec::Vec;

use page_table::addr::{align_down_4k, PhysAddr, VirtAddr};
use page_table::table::PageSize;

use kernel_sync::{Mutex, MutexGuard};
use syscall_define::ipc::{ShmAtFlags, ShmCtlCmd, ShmGetFlags, IPC_PRIVATE};
use syscall_define::LinuxErrno;
use syscall_table::syscall_func;

use crate::config::FRAME_SIZE;
use crate::memory::{frames_alloc, FrameTracker};
use crate::task::current_task;

#[derive(Debug)]
pub struct ShmMemory {
    inner: Mutex<ShmMemoryInner>,
}

#[derive(Debug)]
pub struct ShmMemoryInner {
    ref_count: usize,
    pub frames: Vec<FrameTracker>,
    state: ShmMemoryState,
}

#[derive(Debug, Clone)]
pub struct ShmInfo {
    pub start_va: usize,
    pub end_va: usize,
}

impl ShmInfo {
    pub fn new(start_va: usize, end_va: usize) -> Self {
        Self { start_va, end_va }
    }
}

impl ShmMemory {
    pub fn new(frames: Vec<FrameTracker>) -> Self {
        Self {
            inner: Mutex::new(ShmMemoryInner {
                ref_count: 0,
                frames,
                state: ShmMemoryState::Init,
            }),
        }
    }
    pub fn access_inner(&self) -> MutexGuard<ShmMemoryInner> {
        self.inner.lock()
    }
    pub fn len(&self) -> usize {
        self.access_inner().frames.len() * FRAME_SIZE
    }

    pub fn add_ref(&self) {
        self.access_inner().ref_count += 1;
    }
    pub fn get_ref(&self) -> usize {
        self.access_inner().ref_count
    }
    pub fn delete(&self) {
        self.access_inner().state = ShmMemoryState::Deleted;
    }
    pub fn is_deleted(&self) -> bool {
        self.access_inner().state == ShmMemoryState::Deleted
    }

    pub fn dec_ref(&self) {
        self.access_inner().ref_count -= 1;
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum ShmMemoryState {
    Init,
    Used,
    Deleted,
}

pub static SHM_MEMORY: Mutex<BTreeMap<usize, ShmMemory>> = Mutex::new(BTreeMap::new());

/// 一个系统调用，用于创建一块共享内存，方便进程间通信。
///
/// 参数：
/// + `key`: 指明共享内存的键值，多个进程可以通过它来访问同一个共享内存。当其值为 `IPC_PRIVATE` 时，用于创建当前进程的私有共享内存，多用于父子进程间。
/// + `size`: 用于指明创建共享内存区大小。在函数执行过程中，内核将自动将该值与帧大小(4K)对齐。
/// + `shmflg`: 用于指明操作的类型。当包含 `IPC_CREAT` 时，将创建一块共享内存，目前 Alien 中仅对 `IPC_CREAT` 有所识别。其它 flag 具体可见 [`ShmGetFlags`]。
///
/// 如果已经有共享内存使用了键值 `key`，那么将直接返回 `key` 的值，不会进行创建共享内存操作。
/// 
/// 返回值：如果创建共享内存成功或已经有共享内存使用了键值 `key`，则返回 `key` 值；否则返回 `ENOENT`。
///
/// Reference: [shmget](https://man7.org/linux/man-pages/man2/shmget.2.html)
#[syscall_func(194)]
pub fn shmget(key: usize, size: usize, shmflg: u32) -> isize {
    warn!(
        "shmget key:{},size:{},shmflg:{:?}",
        key,
        size,
        ShmGetFlags::from_bits_truncate(shmflg as i32)
    );
    let key = if key == IPC_PRIVATE {
        // we must create a key
        *SHM_MEMORY.lock().keys().max().unwrap_or(&0) + 1
    } else {
        key
    };
    let mut shm_memory = SHM_MEMORY.lock();
    let shm = shm_memory.get(&key);
    // now we ignore flag
    if shm.is_some() {
        return key as isize;
    }
    let flag = ShmGetFlags::from_bits_truncate(shmflg as i32);
    if flag.contains(ShmGetFlags::IPC_CREAT) {
        warn!("create new share memory {}", key);
        // alloc frames
        let frames = frames_alloc(align_down_4k(size) / FRAME_SIZE);
        if frames.is_none() {
            return LinuxErrno::ENOMEM as isize;
        }
        let frames = frames.unwrap();
        let share_mem = ShmMemory::new(frames);
        shm_memory.insert(key, share_mem);
        return key as isize;
    }
    LinuxErrno::ENOENT as isize
}

/// 一个系统调用，用于将一块共享内存映射到进程的虚拟空间中。通常与 [`shmget`] 一起使用。
///
/// 参数：
/// + `shmid`: 用于指明要映射的共享内存的键值 `key`, 一般为 [`shmget`] 的返回值。
/// + `shmaddr`: 用于指明共享内存要映射到的虚存地址。一般有以下几种情况(目前Alien只能处理情况1，其余情况会导致 panic 退出)
///     1. 如果 `shmaddr` 是NULL，系统将自动选择一个合适的地址
///     2. 如果 `shmaddr` 不是NULL 并且没有指定 SHM_RND，则此段连接到addr所指定的地址上
///     3. 如果 `shmaddr` 不是NULL 并且指定了 SHM_RND，则此段连接到 shmaddr -(shmaddr mod SHMLAB)所表示的地址上
/// + `shmflg`: 一组标志位，通常为0。详细可见 [`ShmAtFlags`]。
///
/// 函数正常执行且映射成功时，则会返回虚拟空间中映射的首地址；当 `shmid` 不合法时，会返回 `EINVAL`。
/// 
/// Reference: [shmat](https://www.man7.org/linux/man-pages/man3/shmat.3p.html)
#[syscall_func(196)]
pub fn shmat(shmid: usize, shmaddr: usize, shmflg: u32) -> isize {
    warn!(
        "shmat shmid:{},shmaddr:{:#x},shmflg:{:?}",
        shmid,
        shmaddr,
        ShmAtFlags::from_bits_truncate(shmflg as i32)
    );
    let shm_memory = SHM_MEMORY.lock();
    let shm = shm_memory.get(&shmid);
    if shm.is_none() {
        return LinuxErrno::EINVAL as isize;
    }
    let shm = shm.unwrap();
    let flag = ShmAtFlags::from_bits_truncate(shmflg as i32);
    assert!(flag.is_empty());
    if flag.contains(ShmAtFlags::SHM_RDONLY) {
        warn!("read only");
    }
    assert_eq!(shmaddr, 0);
    // we must find a place to map
    let task = current_task().unwrap();
    let free_map = task.access_inner().mmap.alloc(shm.len());
    // map to va
    error!("shm map range:{:#x?}", free_map);
    shm.access_inner().state = ShmMemoryState::Used;
    let mut task_inner = task.access_inner();
    let mut address_space = task_inner.address_space.lock();
    // let shm_inner = shm.access_inner();
    let mut virt_start = free_map.start;
    shm.access_inner().frames.iter().for_each(|x| {
        let phy_start = x.start();
        address_space
            .map(
                VirtAddr::from(virt_start),
                PhysAddr::from(phy_start),
                PageSize::Size4K,
                "UVRWAD".into(),
            )
            .unwrap();
        error!("map {:#x} to {:#x}", phy_start, virt_start);
        virt_start += FRAME_SIZE;
    });
    drop(address_space);
    task_inner
        .shm
        .insert(shmid, ShmInfo::new(free_map.start, free_map.end));
    shm.add_ref();
    free_map.start as isize
}

/// 一个系统调用，用于控制共享内存。
///
/// 参数：
/// + `shmid`: 用于指明要操作的共享内存的键值 `key`, 一般为 [`shmget`] 的返回值。
/// + `cmd`: 指明要采取的操作。具体可见 [`ShmCtlCmd`]，目前Alien仅支持 `IpcRmid`，即删除共享内存操作。
/// + `_buf`: 指向一个存储共享内存模式和访问权限的结构，目前未用到。
/// 
/// 当接受的 `cmd` 为 `IpcRmid` 且 成功执行后，将返回 0；否则会因为还未支持相关操作类型而 panic。
///
/// Reference: [shmctl](https://man7.org/linux/man-pages/man2/shmctl.2.html)
#[syscall_func(195)]
pub fn shmctl(shmid: usize, cmd: usize, _buf: usize) -> isize {
    let cmd = ShmCtlCmd::try_from(cmd as u32);
    if cmd.is_err() {
        return LinuxErrno::EINVAL as isize;
    }
    let cmd = cmd.unwrap();
    match cmd {
        ShmCtlCmd::IpcRmid => {
            //delete
            let shm_memory = SHM_MEMORY.lock();
            let shm = shm_memory.get(&shmid);
            if shm.is_none() {
                return LinuxErrno::EINVAL as isize;
            }
            let shm = shm.unwrap();
            shm.delete();
            let task = current_task().unwrap();
            let task_inner = task.access_inner();
            let have_detach = task_inner.shm.get(&shmid).clone();
            if have_detach.is_some() {
                // task_inner.shm.remove(&shmid);
                // unmap
                // let mut address_space = task_inner.address_space.lock();
                // let mut virt_start = shm.access_inner().start_va;
                // let virt_end = shm.access_inner().end_va;
                // error!("shm unmap rang: {:#x}-{:#x}",virt_start,virt_end);
                // while virt_start < virt_end {
                //     let (phy, flag, _) = address_space.query(
                //         VirtAddr::from(virt_start)
                //     ).unwrap();
                //     error!("query {:#x} to {:#x} {:?}",virt_start,phy,flag);
                //     address_space.unmap(VirtAddr::from(virt_start)).unwrap();
                //     virt_start += FRAME_SIZE;
                // }
            }
            let mut flag = false;
            if shm.get_ref() == 0 && shm.is_deleted() {
                flag = true;
            }
            if flag {
                // shm_memory.remove(&shmid);
            }
        }
        _ => {
            panic!("not support")
        }
    }
    0
}
