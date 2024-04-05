use constants::{signal::SignalNumber, task::CloneFlags, AlienError, AlienResult};

use crate::{
    processor::{add_task, current_task},
    scheduler::do_suspend,
    task::CloneArgs,
};

pub fn do_clone(
    flags: usize,
    stack: usize,
    ptid: usize,
    tls: usize,
    ctid: usize,
) -> AlienResult<isize> {
    let clone_flag = CloneFlags::from_bits_truncate(flags as u32);
    // check whether flag include signal
    let sig = flags & 0xff;
    let sig = SignalNumber::from(sig);
    let mut task = current_task().unwrap();
    let child_num = task.inner().children.len();
    if child_num >= 10 {
        do_suspend();
        task = current_task().unwrap();
    }
    let clone_args = CloneArgs {
        flags: clone_flag,
        stack,
        ptid,
        tls,
        ctid,
        sig,
    };
    let new_task = task.do_clone(clone_args).ok_or(AlienError::EAGAIN)?;
    // update return value
    let trap_frame = new_task.trap_frame();
    trap_frame.update_res(0);
    let tid = new_task.tid.raw();
    add_task(new_task);
    Ok(tid as isize)
}
