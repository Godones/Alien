//! 内核启动模块
//!
//! 该模块主要完成以下功能：
//! 1. 初始化内核的栈空间
//! 2. 清空.bss段
//! 3. 从设备树中获取基本的信息
//! 4. 初始化内存子系统
//! 5. 初始化中断子系统
//! 6. 初始化系统调用
//! 7. 初始化进程子系统
//! 8. 初始化文件系统
//! 9. 初始化设备子系统
//! 10. 主核唤醒其它核

#![no_std]
#![no_main]
#![feature(naked_functions)]
#![feature(asm_const)]
#![feature(stmt_expr_attributes)]
#![deny(missing_docs)]

use core::hint::spin_loop;
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use cfg_if::cfg_if;
use kernel::init_init_array;

use arch::hart_id;
use basemachine::machine_info_from_dtb;
#[cfg(not(feature = "qemu"))]
use kernel::board;
#[cfg(any(
    feature = "vf2",
    feature = "qemu",
    feature = "hifive",
    feature = "cv1811h"
))]
use kernel::board::init_dtb;
use kernel::config::CPU_NUM;
use kernel::device::init_device;
use kernel::fs::init_filesystem;
use kernel::interrupt::init_plic;
use kernel::memory::{init_memory_system, kernel_info};
use kernel::print::init_print;
use kernel::sbi::hart_start;
use kernel::{config, init_machine_info, println, task, timer, trap};

mod entry;

/// 多核启动标志
static STARTED: AtomicBool = AtomicBool::new(false);
/// cpu启动计数
static CPUS: AtomicUsize = AtomicUsize::new(0);

extern "C" {
    fn _start();
    fn sbss();
    fn ebss();
}

/// 清空.bss段
#[inline]
fn clear_bss() {
    unsafe {
        core::slice::from_raw_parts_mut(sbss as usize as *mut u8, ebss as usize - sbss as usize)
            .fill(0);
    }
}

/// 内核位于高级语言的入口
#[no_mangle]
pub fn main(_: usize, _: usize) -> ! {
    if !STARTED.load(Ordering::Relaxed) {
        clear_bss();
        println!("{}", config::FLAG);
        cfg_if! {
            if #[cfg(not(feature = "qemu"))] {
                let device_tree_addr = board::FDT.as_ptr() as usize;
            }else{
                use crate::entry::device_tree_addr;
                let device_tree_addr = device_tree_addr();
            }
        }
        init_print();
        println!(
            "boot hart id: {}, device tree addr: {:#x}",
            hart_id(),
            device_tree_addr
        );
        let machine_info = machine_info_from_dtb(device_tree_addr);
        println!("{:#x?}", machine_info);
        init_machine_info(machine_info.clone());
        kernel_info(machine_info.memory.end);
        init_memory_system(machine_info.memory.end, true);
        arch::allow_access_user_memory();
        // init device tree
        #[cfg(feature = "qemu")]
        init_dtb(Some(device_tree_addr));
        #[cfg(feature = "vf2")]
        init_dtb(None);
        // init plic associate board
        init_plic();
        // init all device
        init_device();
        trap::init_trap_subsystem();
        init_filesystem().expect("Init filesystem failed");
        task::init_process();
        // register all syscall
        init_init_array!();
        CPUS.fetch_add(1, Ordering::Release);
        STARTED.store(true, Ordering::Relaxed);
        init_other_hart(hart_id());
    } else {
        while !STARTED.load(Ordering::Relaxed) {
            spin_loop();
        }
        arch::allow_access_user_memory();
        println!("hart {:#x} start", hart_id());
        init_memory_system(0, false);
        trap::init_trap_subsystem();
        CPUS.fetch_add(1, Ordering::Release);
    }
    timer::set_next_trigger();
    println!("Begin run task...");
    task::schedule::first_into_user();
}

/// 唤醒其它核
///
/// 对于qemu来说，只需要工具所有的核都是一样的，因此从严号核开始唤醒。
/// 对于visionfive2/unmatched 来说，0号核只有M态，因此不进行唤醒
fn init_other_hart(hart_id: usize) {
    cfg_if! {
        if #[cfg(any(feature="vf2",feature = "hifive"))]{
            let start_hart = 1;
        }else {
            let start_hart = 0;
        }
    }
    for i in start_hart..CPU_NUM {
        if i != hart_id {
            let res = hart_start(i, _start as usize, 0);
            assert_eq!(res.error, 0);
        }
    }
    wait_all_cpu_start();
}

/// 等待其它cpu核启动
///
/// 对于visionfive2/unmatched 来说，我们使用make SMP=2 来进行单核启动，这里针对这个情况做了处理
fn wait_all_cpu_start() {
    cfg_if! {
        if #[cfg(any(feature="vf2",feature = "hifive"))]{
            let cpu_num = CPU_NUM - 1;
        }else {
            let cpu_num = CPU_NUM;
        }
    }
    while CPUS.load(Ordering::Acquire) < cpu_num {
        spin_loop()
    }
    println!("All cpu start");
}
