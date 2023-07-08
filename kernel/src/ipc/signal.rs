use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::mem::size_of;

use kernel_sync::Mutex;
use syscall_define::signal::{
    SigAction, SigActionDefault, SigActionFlags, SigInfo, SigProcMaskHow, SignalNumber,
    SignalReceivers, SignalUserContext, SimpleBitSet,
};
use syscall_define::LinuxErrno;
use syscall_table::syscall_func;

use crate::task::{current_task, do_exit};
use crate::timer::{sys_nanosleep, TimeSpec};

/// 从 tid 获取信号相关信息
static TID2SIGNALS: Mutex<BTreeMap<usize, Arc<Mutex<SignalReceivers>>>> =
    Mutex::new(BTreeMap::new());

/// 所有线程初始化时均需要加入表
pub fn global_register_signals(tid: usize, signals: Arc<Mutex<SignalReceivers>>) {
    TID2SIGNALS.lock().insert(tid, signals).take();
}

/// 所有线程退出时均需要从表中删除
pub fn global_logoff_signals(tid: usize) {
    TID2SIGNALS.lock().remove(&tid).take();
}

/// 获取信号量。这个函数会复制一个 Arc，不会影响表中的信号本身
pub fn get_signals_from_tid(tid: usize) -> Option<Arc<Mutex<SignalReceivers>>> {
    TID2SIGNALS.lock().get(&tid).map(|s| s.clone())
}

/// 发送一个信号给进程 tid
pub fn send_signal(tid: usize, signum: usize) {
    if let Some(signals) = get_signals_from_tid(tid) {
        // 获取目标线程(可以是自己)的 signals 数组
        warn!("send signal {:?} to {}", SignalNumber::from(signum), tid);
        signals.lock().try_add_bit(signum);
    }
}

#[syscall_func(134)]
pub fn sigaction(sig: usize, action: usize, old_action: usize) -> isize {
    let action = action as *const SigAction;
    let old_action = old_action as *mut SigAction;
    // check whether sig is valid
    let signum = SignalNumber::from(sig);
    if signum == SignalNumber::SIGSTOP
        || signum == SignalNumber::SIGKILL
        || signum == SignalNumber::ERR
    {
        return LinuxErrno::EINVAL as isize;
    }
    let task = current_task().unwrap();
    let task_inner = task.access_inner();
    let mut signal_handler = task_inner.signal_handlers.lock();
    if !old_action.is_null() {
        let ptr = task_inner.transfer_raw(old_action as usize);
        signal_handler.get_action(sig, ptr as *mut SigAction);
    }
    if !action.is_null() {
        let action = task_inner.transfer_raw(action as usize);
        signal_handler.set_action(sig, action as *const SigAction);
    }
    0
}

/// Reference: https://linux.die.net/man/2/sigtimedwait
#[syscall_func(137)]
pub fn sigtimewait(set: usize, info: usize, time: usize) -> isize {
    warn!(
        "sigtimewait: set: {:x}, info: {:x}, time: {:x}",
        set, info, time
    );
    loop {
        let task = current_task().unwrap();
        let task_inner = task.access_inner();
        let mut signal_receivers = task_inner.signal_receivers.lock();
        for i in 1..64 {
            if set & (1 << i) != 0 {
                if signal_receivers.check_signal(i) {
                    if info != 0 {
                        let info = task_inner.transfer_raw_ptr_mut(info as *mut SigInfo);
                        info.si_signo = i as i32;
                        info.si_code = 0;
                    }
                    return i as isize;
                }
            }
        }
        let time_spec = task_inner.transfer_raw_ptr(time as *const TimeSpec);
        // wait time
        if time_spec.tv_sec == 0 && time_spec.tv_nsec == 0 {
            return -1;
        }
        drop(signal_receivers);
        drop(task_inner);
        sys_nanosleep(time as *mut u8, 0 as *mut u8);

        // set the time to 0 to exit the loop
        let task = current_task().unwrap();
        let task_inner = task.access_inner();
        let time_spec = task_inner.transfer_raw_ptr_mut(time as *mut TimeSpec);
        time_spec.tv_sec = 0;
        time_spec.tv_nsec = 0;
    }
}

#[syscall_func(135)]
pub fn sys_sigprocmask(
    how: usize,
    set: *const usize,
    oldset: *mut usize,
    _sig_set_size: usize,
) -> isize {
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
    let mask: Vec<SignalNumber> = signal_receivers.mask.into();
    warn!("after sigprocmask: {:?}", mask);
    0
}

#[syscall_func(999)]
pub fn signal_return() -> isize {
    let task = current_task().unwrap();
    let mut task_inner = task.access_inner();
    let a0 = task_inner.load_trap_frame();
    a0
}

/// The signal handler
pub fn signal_handler() {
    let task = current_task().unwrap();
    let mut task_inner = task.access_inner();
    let receiver = task_inner.signal_receivers.clone();
    let mut receiver = receiver.lock();
    let handler = task_inner.signal_handlers.clone();
    let handler = handler.lock();
    if let Some(signum) = receiver.get_one_signal() {
        let sig = SignalNumber::from(signum);
        match sig {
            SignalNumber::SIGSEGV | SignalNumber::SIGBUS => {
                // we need exit the process
                warn!("task {:?} exit by signal {:?}", task.tid, sig);
                do_exit(-1);
            }
            _ => {
                if let Some(action) = handler.get_action_ref(signum) {
                    // we find the handler
                    if action.is_ignore() {
                        return;
                    }
                    // save the trap context
                    task_inner.save_trap_frame();
                    let trap_contex = task_inner.trap_frame();
                    // modify trap context
                    // set ra to save user's stack
                    trap_contex.regs()[1] = action.get_restorer();
                    //
                    let old_pc = trap_contex.sepc();
                    trap_contex.set_sepc(action.handler);
                    // a0 ==signum
                    trap_contex.regs()[10] = signum;
                    assert_eq!(trap_contex.regs()[10], signum);
                    //
                    let mut sp = trap_contex.regs()[2] - 0x100; // 128
                    if action.flags.contains(SigActionFlags::SA_SIGINFO) {
                        task_inner.signal_set_siginfo = true;
                        // 如果带 SIGINFO，则需要在用户栈上放额外的信息
                        sp = (sp - size_of::<SigInfo>()) & !0xf;
                        info!("add siginfo at {:x}", sp);
                        let mut info = SigInfo::default();
                        info.si_signo = signum as i32;
                        unsafe {
                            *(sp as *mut SigInfo) = info;
                        }
                        // a1 = &siginfo
                        trap_contex.regs()[11] = sp;
                        sp = (sp - size_of::<SignalUserContext>()) & !0xf;
                        unsafe {
                            *(sp as *mut SignalUserContext) =
                                SignalUserContext::init(receiver.mask.bits() as u64, old_pc);
                        }
                        // a2 = &ucontext
                        trap_contex.regs()[12] = sp;
                    }
                    // set sp
                    trap_contex.regs()[2] = sp;
                } else {
                    // find the default handler
                    // 否则，查找默认处理方式
                    match SigActionDefault::of_signal(sig) {
                        SigActionDefault::Terminate => {
                            // 这里不需要 drop(task)，因为当前函数没有用到 task_inner，在 task.save_trap... 内部用过后已经 drop 了
                            drop(task_inner);
                            drop(handler);
                            drop(receiver);
                            do_exit(-1);
                        }
                        SigActionDefault::Ignore => {
                            // 忽略信号时，要将已保存的上下文删除
                            warn!("ignore signal {:?}", sig);
                        }
                    }
                }
            }
        }
    }
}
