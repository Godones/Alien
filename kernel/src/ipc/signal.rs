use syscall_define::LinuxErrno;
use syscall_define::signal::{SigProcMaskHow, SimpleBitSet};
use syscall_table::syscall_func;

use crate::task::current_task;

#[syscall_func(135)]
pub fn sys_sigprocmask(how: usize, set: *const usize, oldset: *mut usize, _sig_set_size: usize) -> isize {
    let task = current_task().unwrap();
    let task_inner = task.access_inner();
    let mut signal_receivers = task_inner.signal_receivers.lock();
    if !oldset.is_null() {
        let set_mut = task_inner.transfer_raw_ptr_mut(oldset);
        *set_mut = signal_receivers.mask.bits();
    }
    if !set.is_null() {
        let set = task_inner.transfer_raw_ptr(set);
        let how = SigProcMaskHow::from(how);
        match how {
            SigProcMaskHow::SigBlock => {
                signal_receivers.mask += SimpleBitSet::from(*set);
            }
            SigProcMaskHow::SigUnblock => {
                signal_receivers.mask -= SimpleBitSet::from(*set);
            }
            SigProcMaskHow::SigSetMask => {
                signal_receivers.mask = SimpleBitSet::from(*set);
            }
            SigProcMaskHow::Unknown => {
                return LinuxErrno::EINVAL as isize;
            }
        }
    }
    0
}