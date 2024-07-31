use constants::{sys::PrctlOp, AlienResult};

use crate::task::current_task;

#[syscall_func(167)]
pub fn prctl(op: u32, _arg2: u64, _arg3: u64, _arg4: u64, _arg5: u64) -> AlienResult<isize> {
    let op = PrctlOp::try_from(op).unwrap();
    match op {
        PrctlOp::PR_SET_NAME => {
            let name_ptr = _arg2 as *const u8;
            let task = current_task().unwrap();
            let str = task.transfer_str(name_ptr);
            // println_color!(32, "prctl: set task name: {}", str);
            task.access_inner().set_name(str);
            Ok(0)
        }
        op => panic!("prctl: op {:?} not implemented", op),
    }
}
