#[cfg(feature = "kprobe_test")]
pub mod kprobe_test;

use alloc::{sync::Arc, vec::Vec};
use core::alloc::Layout;

use config::FRAME_SIZE;
use kprobe::{
    Kprobe, KprobeAuxiliaryOps, KprobeBuilder, KprobeManager, KprobePointList, KretprobeInstance,
};
use ksync::Mutex;
use mem::{alloc_kernel_free_region, kernel_space};
use page_table::{addr::VirtAddr, pte::MappingFlags};

use crate::{task::current_task, trap::CommonTrapFrame};

pub type KernelKprobe = Kprobe<Mutex<()>, KprobeAuxiliary>;

#[derive(Debug)]
pub struct KprobeAuxiliary;

impl KprobeAuxiliaryOps for KprobeAuxiliary {
    fn set_writeable_for_address(address: usize, len: usize, writable: bool) {
        assert!(len < FRAME_SIZE);
        let kspace = kernel_space();
        let mut guard = kspace.lock();
        let (_phy_addr, flag, _size) = guard.query(VirtAddr::from(address)).unwrap();
        // println_color!(31,"set_writeable_for_address: virt_addr:{:#x} -> phy_addr: {:#x}, flag: {:#x}, size: {:?}", address,phy_addr, flag, size);
        let new_flag = if writable {
            flag | MappingFlags::W
        } else {
            flag & !MappingFlags::W
        };
        guard
            .modify_pte_flags(VirtAddr::from(address), new_flag, false)
            .unwrap();
    }

    fn alloc_executable_memory(layout: Layout) -> *mut u8 {
        let kspace = kernel_space();
        let mut guard = kspace.lock();
        assert!(layout.size() < FRAME_SIZE);
        let region_start = alloc_kernel_free_region(FRAME_SIZE);
        guard
            .map_region_no_target(
                VirtAddr::from(region_start),
                FRAME_SIZE,
                MappingFlags::from("RWXVAD"),
                false,
                false,
            )
            .unwrap();
        region_start as *mut u8
    }

    fn dealloc_executable_memory(ptr: *mut u8, _layout: Layout) {
        let kspace = kernel_space();
        let mut guard = kspace.lock();
        let region_start = ptr as usize;
        guard
            .unmap_region(VirtAddr::from(region_start), FRAME_SIZE)
            .unwrap();
    }

    fn insert_kretprobe_instance_to_task(instance: kprobe::KretprobeInstance) {
        static INSTANCE: Mutex<Vec<KretprobeInstance>> = Mutex::new(Vec::new());
        let task = current_task();
        if let Some(task) = task {
            let mut inner = task.access_inner();
            inner.kretprobe_instances.push(instance);
        } else {
            // If the current task is None, we can store it in a static variable
            let mut instances = INSTANCE.lock();
            instances.push(instance);
        }
    }

    fn pop_kretprobe_instance_from_task() -> kprobe::KretprobeInstance {
        static INSTANCE: Mutex<Vec<KretprobeInstance>> = Mutex::new(Vec::new());
        let task = current_task();
        if let Some(task) = task {
            let mut inner = task.access_inner();
            inner.kretprobe_instances.pop().unwrap()
        } else {
            // If the current task is None, we can pop it from the static variable
            let mut instances = INSTANCE.lock();
            instances.pop().unwrap()
        }
    }
}

pub static KPROBE_MANAGER: Mutex<KprobeManager<Mutex<()>, KprobeAuxiliary>> =
    Mutex::new(KprobeManager::new());
static KPROBE_POINT_LIST: Mutex<KprobePointList<KprobeAuxiliary>> =
    Mutex::new(KprobePointList::new());

/// Unregister a kprobe
pub fn unregister_kprobe(kprobe: Arc<KernelKprobe>) {
    let mut manager = KPROBE_MANAGER.lock();
    let mut kprobe_list = KPROBE_POINT_LIST.lock();
    kprobe::unregister_kprobe(&mut manager, &mut kprobe_list, kprobe);
}

/// Register a kprobe
pub fn register_kprobe(kprobe_builder: KprobeBuilder<KprobeAuxiliary>) -> Arc<KernelKprobe> {
    let mut manager = KPROBE_MANAGER.lock();
    let mut kprobe_list = KPROBE_POINT_LIST.lock();
    let kprobe = kprobe::register_kprobe(&mut manager, &mut kprobe_list, kprobe_builder);
    kprobe
}

pub fn run_all_kprobe(frame: &mut CommonTrapFrame) -> Option<()> {
    let mut manager = KPROBE_MANAGER.lock();
    let mut pt_regs = frame.to_pt_regs();
    let res = kprobe::kprobe_handler_from_break(&mut manager, &mut pt_regs);
    frame.update_from_pt_regs(&pt_regs);
    res
}
