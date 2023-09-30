//! 信号是进程间通信机制中唯一的异步通信机制，进程之间可以互相通过系统调用 kill 发送软中断信号。
//! 内核也可以因为内部事件而给进程发送信号，通知进程发生了某个事件。
//!
//! 有关 Alien 中信号的具体处理流程可见 [`signal_handler`]。
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::mem::size_of;

use crate::ksync::Mutex;
use syscall_define::signal::{
    SigAction, SigActionDefault, SigActionFlags, SigInfo, SigProcMaskHow, SignalNumber,
    SignalReceivers, SignalUserContext, SimpleBitSet,
};
use syscall_define::LinuxErrno;
use syscall_table::syscall_func;

use crate::task::{current_task, do_exit, do_suspend};
use crate::timer::{read_timer, TimeSpec};

/// 记录每个线程的信号量，从 tid 获取信号相关信息
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

/// 一个系统调用，用于获取或修改与指定信号相关联的处理动作。
///
/// 一个进程，对于每种信号，在不进行特殊设置的情况下，都有其默认的处理方式。有关信号的处理流程具体可见 [`signal_handler`] 与 [`SigActionDefault`]。
/// 用户可以通过 `sigaction` 获取或修改进程在接收到某信号时的处理动作。
///
/// 参数：
/// + `sig`: 指出要修改的处理动作所捕获的信号类型。有关详情可见 [`SignalNumber`]。
/// + `action`: 指定新的信号处理方式的指针。详情可见 [`SigAction`]。当该值为空指针时，`sigaction` 将不会修改信号的处理动作。
/// + `old_action`: 指出原信号处理方式要保存到的位置。详情可见 [`SigAction`]。当该值为空指针时，`sigaction` 将不会保存信号的原处理动作。
///
/// 函数执行成功后返回 0；若输入的 `sig` 是 `SIGSTOP`, `SIGKILL`, `ERR`中的一个时，将导致函数返回 `EINVAL`。
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
    let mut task_inner = task.access_inner();
    let signal_handler = task_inner.signal_handlers.clone();
    let mut signal_handler = signal_handler.lock();
    if !old_action.is_null() {
        let mut tmp = SigAction::empty();
        signal_handler.get_action(sig, &mut tmp);
        task_inner.copy_to_user(&tmp, old_action);
    }
    if !action.is_null() {
        let mut tmp_action = SigAction::empty();
        task_inner.copy_from_user(action, &mut tmp_action);
        warn!("sig {:?} action is {:?}", signum, tmp_action);
        signal_handler.set_action(sig, &tmp_action);
    }
    0
}

/// 一个系统调用，用于使得一个进程在一段时间限制内等待一个信号，并保存信号的相关信息。
///
/// 参数：
/// + `set`: 用于指明等待的信号集，当进程接收到 `set` 中的任一一种信号时，都会返回。
/// + `info`: 用于指明保存信号相关信息的位置。 当该值为空时，将不执行保存信号信息的操作。具体可见 [`SigInfo`] 结构。
/// + `time`: 指明等待的时间。具体可见 [`TimeSpec`] 结构。
///
/// 当函数在规定的时间内成功接收到 `set` 中包含的某个信号时，将会返回该信号的序号；
/// 当函数在规定的时间内未接收到 `set` 中包含的某个信号时，将返回 `EAGAIN` 表示超时；
/// 如果 `time` 所指明的时间为 0，那么函数将直接返回-1。
///
/// Reference: [sigtimedwait](https://linux.die.net/man/2/sigtimedwait)
#[syscall_func(137)]
pub fn sigtimewait(set: usize, info: usize, time: usize) -> isize {
    warn!(
        "sigtimewait: set: {:x}, info: {:x}, time: {:x}",
        set, info, time
    );

    let mut flag = false;
    let mut target_time = 0;

    let task = current_task().unwrap().clone();
    let mut time_spec = TimeSpec::new(0, 0);
    task.access_inner()
        .copy_from_user(time as *const TimeSpec, &mut time_spec);
    loop {
        let mut task_inner = task.access_inner();
        let mut signal_receivers = task_inner.signal_receivers.lock();
        for i in 1..64 {
            if set & (1 << i) != 0 {
                if signal_receivers.check_signal(i) {
                    if info != 0 {
                        let mut tmp_info = SigInfo::default();
                        tmp_info.si_signo = i as i32;
                        tmp_info.si_code = 0;
                        drop(signal_receivers);
                        task_inner.copy_to_user(&tmp_info, info as *mut SigInfo);
                    }
                    return i as isize;
                }
            }
        }

        // wait time
        if time_spec.tv_sec == 0 && time_spec.tv_nsec == 0 {
            return -1;
        }
        drop(signal_receivers);
        drop(task_inner);
        if !flag {
            warn!("sigtimewait: sleep for {:?}", time_spec);
            let t_time = read_timer() + time_spec.to_clock();
            target_time = t_time;
            flag = true;
        }
        let now = read_timer();
        if now >= target_time {
            warn!("sigtimewait: timeout");
            break;
        }
        do_suspend();

        // interrupt by signal
        let task_inner = task.access_inner();
        let receiver = task_inner.signal_receivers.lock();
        if receiver.have_signal() {
            let sig = receiver.have_signal_with_number().unwrap();
            return sig as isize;
        }
    }
    LinuxErrno::EAGAIN.into()
}

/// 一个系统调用，用于获取和设置信号的屏蔽位。通过 `sigprocmask`，进程可以方便的屏蔽某些信号。
///
/// 参数：
/// + `how`: 指明将采取何种逻辑修改信号屏蔽位。大致包括：屏蔽 `set` 中指明的所有信号，将 `set` 中指明的所有信号解除屏蔽或者直接使用 `set` 作为屏蔽码。具体可见 [`SigProcMaskHow`]。
/// + `set`: 用于指明将要修改的信号屏蔽位。具体可见 [`SimpleBitSet`]。当该值为 null 时，将不修改信号的屏蔽位。
/// + `oldset`: 用于获取当前对信号的屏蔽位。具体可见 [`SimpleBitSet`]。当该值为 null 时，将不保存信号的旧屏蔽位。
/// + `_sig_set_size`: 用于指示 `set` 和 `oldset` 所指向的信号屏蔽位的长度，目前在 Alien 中未使用。
///
/// 函数正常执行后，返回 0。
///
/// Reference: [sigprocmask](https://www.man7.org/linux/man-pages/man2/sigprocmask.2.html)
#[syscall_func(135)]
pub fn sigprocmask(how: usize, set: usize, oldset: usize, _sig_set_size: usize) -> isize {
    let task = current_task().unwrap();
    let task_inner = task.access_inner();
    let mut signal_receivers = task_inner.signal_receivers.lock();
    if oldset != 0 {
        let set_mut = task_inner.transfer_raw_ptr_mut(oldset as *mut usize);
        *set_mut = signal_receivers.mask.bits();
    }
    let how = SigProcMaskHow::from(how);
    warn!("sigprocmask: how: {:?}, set: {:x}", how, set);
    if set != 0 {
        let set = task_inner.transfer_raw_ptr(set as *const usize);
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
    trace!("after sigprocmask: {:?}", mask);
    0
}

/// 一个系统调用函数，向 `pid` 指定的进程发送信号。
/// 如果进程中有多个线程，则会发送给任意一个未阻塞的线程。
///
/// pid 有如下情况
/// 1. pid > 0，则发送给指定进程
/// 2. pid = 0，则发送给所有同组进程
/// 3. pid = -1，则发送给除了初始进程(pid=1)外的所有当前进程有权限的进程
/// 4. pid < -2，则发送给组内 pid 为参数相反数的进程
///
/// 目前 2/3/4 未实现。对于 1，仿照 zCore 的设置，认为**当前进程自己或其直接子进程** 是"有权限"或者"同组"的进程。
///  
/// 目前如果函数成功执行后会返回0；否则返回错误类型。
///
/// Reference: [kill](https://man7.org/linux/man-pages/man2/kill.2.html)
#[syscall_func(129)]
pub fn kill(pid: usize, sig: usize) -> isize {
    warn!("kill pid {}, signal id {:?}", pid, SignalNumber::from(sig));
    if pid > 0 {
        //println!("kill pid {}, signal id {}", pid, signal_id);
        if sig > 0 {
            send_signal(pid, sig);
        }
        0
    } else if pid == 0 {
        LinuxErrno::ESRCH as isize
    } else {
        // 如果 signal_id == 0，则仅为了检查是否存在对应进程，此时应该返回参数错误。是的，用户库是会刻意触发这个错误的
        LinuxErrno::EINVAL as isize
    }
}

/// 一个系统调用函数，向 `tid` 指定的线程发送信号。在`Alien`中`tid`是task的唯一标识，故 `tid` 只会指向一个线程。
///
/// 函数正常执行后会返回0；否则返回错误类型。
///
/// Reference: [tkill](https://man7.org/linux/man-pages/man2/tkill.2.html)
#[syscall_func(130)]
pub fn tkill(tid: usize, sig: usize) -> isize {
    warn!("tkill tid {}, signal id {:?}", tid, SignalNumber::from(sig));
    if tid > 0 && sig > 0 {
        //println!("kill pid {}, signal id {}", pid, signal_id);
        send_signal(tid, sig);
        0
    } else {
        // 如果 signal_id == 0，则仅为了检查是否存在对应进程，此时应该返回参数错误。是的，用户库是会刻意触发这个错误的
        LinuxErrno::EINVAL as isize
    }
}

/// 一个系统调用函数，用于在用户态执行完信号处理函数后重新装回原 trap 上下文，一般不会被用户态程序调用。函数返回原 trap 上下文的 a0。
#[syscall_func(139)]
pub fn signal_return() -> isize {
    let task = current_task().unwrap();
    let mut task_inner = task.access_inner();
    let a0 = task_inner.load_trap_frame();
    a0
}

/// 信号处理函数。该函数在进程即将从内核态回到用户态时被调用，用于处理当前进程所接收到的信号。
///
/// 进行信号处理的前提:
/// 1. 有要处理的信号；
/// 2. 该信号目前没有被该进程屏蔽；
/// 3. 该信号没有被当前正在处理的信号屏蔽。
///
/// 当进入 `signal_handler` 后，对于该进程 `signal_receivers` 下所有信号种类开始遍历：
/// 先检查此种信号是否满足上面所有的前提，如果有一项以上不满足，直接continue;
/// 否则需要根据该信号是否已经设置非默认的处理函数进行接下来的操作。
///
/// + 对于一些固定采用采用默认信号处理方式的信号，或由于未设置其它信号处理函数的信号，仍然使用默认信号处理方式，Alien 中采用 [`SigActionDefault`] 对该信号进行判定：
///     + 如果属于 `Terminate` 类型，将导致进程终止。
///     + 如果属于 `Ignore` 类型，进程将直接忽略该信号。
/// + 如果进程已经设置过信号处理函数，由于信号处理函数的位置位于用户虚拟内存空间，需要回到用户态下进行信号处理函数的执行，
/// 但由于原来在用户态下我们还保存有一个 trap 上下文，因此我们需要记录这个 trap 上下文，同时将设计好的新的执行信号处理函数的上下文转移至原trap上下文的位置，
/// 以便其执行用户态下的信号处理函数。
///
/// 待用户态下的信号处理函数执行完毕后进程将重新陷入内核态，调用 [`signal_return`] 重新装载回原 trap 上下文。
/// 至此，一个信号被处理完毕。
pub fn signal_handler() {
    let task = current_task().unwrap();
    let mut task_inner = task.access_inner();
    let receiver = task_inner.signal_receivers.clone();
    let mut receiver = receiver.lock();
    let handler = task_inner.signal_handlers.clone();
    let handler = handler.lock();
    if let Some(signum) = receiver.get_one_signal() {
        let sig = SignalNumber::from(signum);
        error!("task {:?} receive signal {:?}", task.tid, sig);
        match sig {
            SignalNumber::SIGSEGV | SignalNumber::SIGBUS => {
                // we need exit the process
                drop(task_inner);
                drop(handler);
                drop(receiver);
                warn!("task {:?} exit by signal {:?}", task.tid, sig);
                do_exit(-1);
            }
            _ => {
                if let Some(action) = handler.get_action_ref(signum) {
                    // we find the handler
                    if action.is_ignore() {
                        return;
                    }
                    warn!("find handler for signal {:?}", sig);
                    if !task_inner.save_trap_frame() {
                        // we are in signal handler,don't nest
                        return;
                    }
                    // save the trap context
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

                    warn!(
                        "task {:?} handle signal {:?} at {:#x}, old pc: {:#x}, old_sp: {:#x}",
                        task.tid,
                        sig,
                        trap_contex.sepc(),
                        old_pc,
                        trap_contex.regs()[2]
                    );
                    let mut sp = trap_contex.regs()[2] - 0x200; // 128
                    if action.flags.contains(SigActionFlags::SA_SIGINFO) {
                        task_inner.signal_set_siginfo = true;
                        // 如果带 SIGINFO，则需要在用户栈上放额外的信息
                        sp = (sp - size_of::<SigInfo>()) & !0xf;
                        info!("add siginfo at {:x}", sp);
                        let mut info = SigInfo::default();
                        info.si_signo = signum as i32;
                        unsafe {
                            let phy_sp = task_inner.transfer_raw(sp);
                            *(phy_sp as *mut SigInfo) = info;
                        }
                        // a1 = &siginfo
                        trap_contex.regs()[11] = sp;
                        sp = (sp - size_of::<SignalUserContext>()) & !0xf;
                        info!("add ucontext at {:x}", sp);
                        unsafe {
                            let phy_sp = task_inner.transfer_raw(sp);
                            *(phy_sp as *mut SignalUserContext) =
                                SignalUserContext::init(receiver.mask.bits() as u64, old_pc);
                        }
                        // a2 = &ucontext
                        trap_contex.regs()[12] = sp;
                    }
                    // set sp
                    trap_contex.regs()[2] = sp;
                    warn!(
                        "task {:?} handle signal {:?}, pc:{:#x}, sp:{:#x}",
                        task.tid,
                        sig,
                        trap_contex.sepc(),
                        trap_contex.regs()[2]
                    );
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

/// 一个系统调用函数，用于阻塞当前进程，等待其他进程传入信号打断阻塞。当进程接收到某种信号时，终止阻塞，函数返回 `EINTR`。
#[syscall_func(133)]
pub fn sigsuspend() -> isize {
    loop {
        do_suspend();
        let task = current_task().unwrap();
        // interrupt by signal
        let task_inner = task.access_inner();
        let receiver = task_inner.signal_receivers.lock();
        if receiver.have_signal() {
            return LinuxErrno::EINTR.into();
        }
    }
}
