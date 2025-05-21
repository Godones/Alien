#[cfg(feature = "kprobe_test")]
pub mod kprobe_test;

use alloc::sync::Arc;
use core::alloc::Layout;

use config::FRAME_SIZE;
use kprobe::{Kprobe, KprobeAuxiliaryOps, KprobeBuilder, KprobeManager, KprobePointList};
use ksync::Mutex;
use mem::{alloc_kernel_free_region, kernel_space};
use page_table::{addr::VirtAddr, pte::MappingFlags};

use crate::trap::CommonTrapFrame;

pub type KernelKprobe = Kprobe<Mutex<()>, KprobeAuxiliary>;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct KProbeContext {
    pub pc: usize,
    pub ra: usize,
    pub sp: usize,
    pub gp: usize,
    pub tp: usize,
    pub t0: usize,
    pub t1: usize,
    pub t2: usize,
    pub s0: usize,
    pub s1: usize,
    pub a0: usize,
    pub a1: usize,
    pub a2: usize,
    pub a3: usize,
    pub a4: usize,
    pub a5: usize,
    pub a6: usize,
    pub a7: usize,
    pub s2: usize,
    pub s3: usize,
    pub s4: usize,
    pub s5: usize,
    pub s6: usize,
    pub s7: usize,
    pub s8: usize,
    pub s9: usize,
    pub s10: usize,
    pub s11: usize,
    pub t3: usize,
    pub t4: usize,
    pub t5: usize,
    pub t6: usize,
}

impl From<&CommonTrapFrame> for KProbeContext {
    fn from(value: &CommonTrapFrame) -> Self {
        KProbeContext {
            pc: value.pc(),
            ra: value.regs()[1],   // x1
            sp: value.regs()[2],   // x2
            gp: value.regs()[3],   // x3
            tp: value.regs()[4],   // x4
            t0: value.regs()[5],   // x5
            t1: value.regs()[6],   // x6
            t2: value.regs()[7],   // x7
            s0: value.regs()[8],   // x8
            s1: value.regs()[9],   // x9
            a0: value.regs()[10],  // x10
            a1: value.regs()[11],  // x11
            a2: value.regs()[12],  // x12
            a3: value.regs()[13],  // x13
            a4: value.regs()[14],  // x14
            a5: value.regs()[15],  // x15
            a6: value.regs()[16],  // x16
            a7: value.regs()[17],  // x17
            s2: value.regs()[18],  // x18
            s3: value.regs()[19],  // x19
            s4: value.regs()[20],  // x20
            s5: value.regs()[21],  // x21
            s6: value.regs()[22],  // x22
            s7: value.regs()[23],  // x23
            s8: value.regs()[24],  // x24
            s9: value.regs()[25],  // x25
            s10: value.regs()[26], // x26
            s11: value.regs()[27], // x27
            t3: value.regs()[28],  // x28
            t4: value.regs()[29],  // x29
            t5: value.regs()[30],  // x30
            t6: value.regs()[31],  // x31
        }
    }
}

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
    kprobe::kprobe_handler_from_break(&mut manager, frame)
}
