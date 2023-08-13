use alloc::vec::Vec;

use syscall_define::io::{PollEvents, PollFd};
use syscall_define::LinuxErrno;
use syscall_table::syscall_func;

use crate::fs::file::FilePollExt;
use crate::task::{current_task, do_suspend};
use crate::timer::TimeSpec;

/// Reference:https://man7.org/linux/man-pages/man2/ppoll.2.html
#[syscall_func(73)]
pub fn ppoll(fds_ptr: usize, nfds: usize, time: usize, _mask: usize) -> isize {
    // assert_eq!(time, 0);
    // assert_eq!(mask, 0);
    let task = current_task().unwrap();
    let mut fds = Vec::<PollFd>::with_capacity(nfds);
    task.access_inner()
        .copy_from_user_buffer(fds_ptr as *const PollFd, fds.as_mut_ptr(), nfds);
    unsafe {
        fds.set_len(nfds);
    }
    warn!("fds: {:?}", fds);
    let wait_time = if time != 0 {
        let time_spec = task.transfer_raw_ptr(time as *mut TimeSpec);
        Some(time_spec.to_clock() + TimeSpec::now().to_clock())
    } else {
        // wait forever
        None
    };
    let mut res = 0;
    loop {
        let task = current_task().unwrap();
        fds.iter_mut().for_each(|pfd| {
            if let Some(file) = task.get_file(pfd.fd as usize) {
                let mut event = PollEvents::empty();
                if file.in_exceptional_conditions() {
                    event |= PollEvents::ERR;
                }
                if file.is_hang_up() {
                    event |= PollEvents::HUP;
                }
                if pfd.events.contains(PollEvents::IN) && file.ready_to_read() {
                    event |= PollEvents::IN;
                }
                if pfd.events.contains(PollEvents::OUT) && file.ready_to_write() {
                    event |= PollEvents::OUT;
                }
                if !event.is_empty() {
                    res += 1;
                }
                error!("[ppoll]: event: {:?}", event);
                pfd.revents = event;
            } else {
                pfd.events = PollEvents::ERR;
            }
        });
        if res > 0 {
            // copy to user
            task.access_inner()
                .copy_to_user_buffer(fds.as_ptr(), fds_ptr as *mut PollFd, nfds);
            error!("ppoll return {:?}", fds);
            return res;
        }
        if let Some(wait_time) = wait_time {
            if wait_time <= TimeSpec::now().to_clock() {
                error!("ppoll timeout");
                return 0;
            }
        }
        warn!("[poll] suspend");
        // suspend
        do_suspend();
        // interrupt by signal
        let task = current_task().unwrap();
        let task_inner = task.access_inner();
        let receiver = task_inner.signal_receivers.lock();
        if receiver.have_signal() {
            return LinuxErrno::EINTR.into();
        }
    }
}
