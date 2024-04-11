use alloc::sync::Arc;

use arch::interrupt_enable;
use constants::{AlienError, AlienResult};
use context::TrapFrameRaw;
use platform::println;
use riscv::register::scause::{Exception, Trap};

use crate::{SYSCALL_DOMAIN, TASK_DOMAIN};

/// 系统调用异常处理
pub fn syscall_exception_handler() {
    // enable interrupt
    // interrupt_enable();
    let task_domain = TASK_DOMAIN.get().unwrap();
    let trap_frame_phy_addr = task_domain.trap_frame_phy_addr().unwrap();
    let cx = TrapFrameRaw::from_raw_ptr(trap_frame_phy_addr as _);

    cx.update_sepc(cx.sepc() + 4);
    // // get system call return value
    let parameters = cx.parameters();
    let syscall_name = constants::syscall_name(parameters[0]);
    //
    // let task = current_task().unwrap();
    // let p_name = task.get_name();
    // let tid = task.get_tid();
    // let pid = task.get_pid();
    // if !p_name.contains("shell") && !p_name.contains("init") && !p_name.contains("ls") {
    //     // ignore shell and init
    //     info!(
    //         "[pid:{}, tid: {}][p_name: {}] syscall: [{}] {}({:#x}, {:#x}, {:#x}, {:#x}, {:#x}, {:#x})",
    //         pid,
    //         tid,
    //         p_name,
    //         parameters[0],
    //         syscall_name,
    //         parameters[1],
    //         parameters[2],
    //         parameters[3],
    //         parameters[4],
    //         parameters[5],
    //         parameters[6]
    //     );
    // }
    //

    let result = SYSCALL_DOMAIN.get().unwrap().call(
        parameters[0],
        [
            parameters[1],
            parameters[2],
            parameters[3],
            parameters[4],
            parameters[5],
            parameters[6],
        ],
    );

    // cx is changed during sys_exec, so we have to call it again
    let trap_frame_phy_addr = task_domain.trap_frame_phy_addr().unwrap();
    let cx = TrapFrameRaw::from_raw_ptr(trap_frame_phy_addr as _);
    // println!("syscall [{}] result: {:x?}, sepc: {}",syscall_name, result,cx.sepc);

    // if !p_name.contains("shell") && !p_name.contains("init") && !p_name.contains("ls") {
    //     info!(
    //         "[pid:{}, tid: {}] syscall: [{}] result: {:?}, tp: {:#x}",
    //         pid,
    //         tid,
    //         syscall_name,
    //         result,
    //         cx.regs()[4]
    //     );
    // }

    cx.update_res(result.unwrap() as usize);
}
