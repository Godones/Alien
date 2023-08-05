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
