#[cfg(feature = "kprobe_test")]
pub mod kprobe_test;

use core::alloc::Layout;

use config::FRAME_SIZE;
use kprobe::{KprobeAuxiliaryOps, KprobeManager, KprobePointList};
use ksync::Mutex;
use mem::{alloc_kernel_free_region, kernel_space};
use page_table::{addr::VirtAddr, pte::MappingFlags};

use crate::trap::CommonTrapFrame;

#[derive(Debug)]
pub struct KprobeAuxiliaryOpsImpl;

impl KprobeAuxiliaryOps for KprobeAuxiliaryOpsImpl {
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
}

pub static KPROBE_MANAGER: Mutex<KprobeManager<Mutex<()>, KprobeAuxiliaryOpsImpl>> =
    Mutex::new(KprobeManager::new());
static KPROBE_POINT_LIST: Mutex<KprobePointList<KprobeAuxiliaryOpsImpl>> =
    Mutex::new(KprobePointList::new());

pub fn run_all_kprobe(frame: &mut CommonTrapFrame) -> Option<()> {
    let mut manager = KPROBE_MANAGER.lock();
    kprobe::kprobe_handler_from_break(&mut manager, frame)
}
