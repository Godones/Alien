use basic::{sync::OnceGet, task::TrapFrame};
use mem::PhysAddr;

use crate::{syscall_domain, task_domain};

/// 系统调用异常处理
pub fn syscall_exception_handler() {
    let task_domain = task_domain!();
    let tid = crate::task::current_tid();
    let trap_frame_phy_addr = task_domain.trap_frame_phy_addr().unwrap();
    let cx = TrapFrame::from_raw_phy_ptr(PhysAddr::from(trap_frame_phy_addr));

    cx.update_sepc(cx.sepc() + 4);
    let parameters = cx.parameters();
    let _syscall_name = pconst::syscall_name(parameters[0]);

    // let forbid_call = [72, 124, 260, 73, 63, 66];

    let allow_call = [17, 49];

    // if !forbid_call.contains(&parameters[0] ){
    //     log::error!(
    //         "[{:?}] syscall {:?} parameters: {:x?}",
    //         tid,
    //         _syscall_name,
    //         &parameters[1..7]
    //     );
    // }
    if allow_call.contains(&parameters[0]) {
        log::error!(
            "[{:?}] syscall {:?} parameters: {:x?}",
            tid,
            _syscall_name,
            &parameters[1..7]
        );
    }

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
    let res = result.unwrap_or_else(|err| {
        error!("syscall error: {:?}", err);
        err as isize
    });
    if parameters[0] != 72 && parameters[0] != 124 {
        debug!(
            "[tid:{:?}] syscall {:?} result: {:?}",
            tid, _syscall_name, res
        );
    }
    debug!("syscall {:?} result: {:?}", _syscall_name, result);
    cx.update_result(res as usize);
}
