use arch::interrupt_enable;
use basic::task::TrapFrame;
use mem::PhysAddr;

use crate::{syscall_domain, task_domain};

/// 系统调用异常处理
pub fn syscall_exception_handler() {
    // enable interrupt
    interrupt_enable();
    let task_domain = task_domain!();
    let trap_frame_phy_addr = task_domain.trap_frame_phy_addr().unwrap();
    let cx = TrapFrame::from_raw_phy_ptr(PhysAddr::from(trap_frame_phy_addr));

    cx.update_sepc(cx.sepc() + 4);
    let parameters = cx.parameters();
    let _syscall_name = constants::syscall_name(parameters[0]);

    let result = syscall_domain!().call(
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
    let cx = TrapFrame::from_raw_phy_ptr(PhysAddr::from(trap_frame_phy_addr));
    // debug!("syscall [{}] result: {:x?}, sepc: {}",syscall_name, result,cx.sepc);
    cx.update_result(result.unwrap() as usize);
}
