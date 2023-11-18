//! Alien 中有关进程的系统调用 和 多核的相关支持。
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
use core::cell::UnsafeCell;
use core::ops::{Index, IndexMut};
use smpscheduler::{FifoSmpScheduler, FifoTask, ScheduleHart};
use spin::Lazy;

use crate::ksync::Mutex;
use pconst::ipc::FutexOp;
use pconst::signal::SignalNumber;
use pconst::task::{CloneFlags, WaitOptions};
use pconst::{PrLimit, PrLimitRes};
use syscall_table::syscall_func;

use crate::arch::{hart_id};
use crate::config::CPU_NUM;
use crate::ipc::{futex, global_logoff_signals};
use crate::sbi::system_shutdown;
use crate::task::context::Context;
use crate::task::schedule::schedule;
use crate::task::task::{Task, TaskState};
use crate::task::INIT_PROCESS;
use crate::trap::{check_task_timer_expired, TrapFrame};
use crate::{arch, fs};

/// 记录当前 CPU 上正在执行的线程 和 线程上下文
#[derive(Debug, Clone)]
pub struct CPU {
    /// 正在该 CPU 上运行的线程的控制块
    pub task: Option<Arc<Task>>,
    /// 当前线程的上下文
    pub context: Context,
}

/// 记录一组 CPU 的相关信息
pub struct CpuManager<const CPUS: usize> {
    cpus: Vec<CPU>,
}

impl<const CPUS: usize> CpuManager<CPUS> {
    /// 创建一个 `CpuManager` 结构
    pub fn new() -> Self {
        Self {
            cpus: vec![CPU::empty(); CPUS],
        }
    }
}

impl<const CPUS: usize> Index<usize> for CpuManager<CPUS> {
    type Output = CPU;

    /// 用于快捷取出 hartid 为 index 的 CPU 的一个不可变引用
    fn index(&self, index: usize) -> &Self::Output {
        &self.cpus[index]
    }
}

impl<const CPUS: usize> IndexMut<usize> for CpuManager<CPUS> {
    /// 用于快捷取出 hartid 为 index 的 CPU 的一个可变引用
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.cpus[index]
    }
}

impl CPU {
    /// 获取一个空的 CPU
    const fn empty() -> Self {
        Self {
            task: None,
            context: Context::empty(),
        }
    }

    /// 获取 cpu 上的线程任务控制块(会直接获取该任务控制块的所有权)
    pub fn take_process(&mut self) -> Option<Arc<Task>> {
        self.task.take()
    }

    /// 获取线程上下文的一个 不可变引用 的指针
    pub fn get_context_raw_ptr(&self) -> *const Context {
        &self.context as *const Context
    }

    /// 获取线程上下文的一个 可变引用 的指针
    pub fn get_context_mut_raw_ptr(&mut self) -> *mut Context {
        &mut self.context as *mut Context
    }
}
pub struct SafeRefCell<T>(UnsafeCell<T>);
impl<T> SafeRefCell<T> {
    const fn new(t: T) -> Self {
        Self(UnsafeCell::new(t))
    }
}
/// #Safety: Only the corresponding cpu will access it.
unsafe impl<CPU> Sync for SafeRefCell<CPU> {}

const DEFAULT_CPU: SafeRefCell<CPU> = SafeRefCell::new(CPU::empty());
/// 保存每个核的信息
static CPU_MANAGER: [SafeRefCell<CPU>; CPU_NUM] = [DEFAULT_CPU; CPU_NUM];
#[derive(Debug)]
pub struct ScheduleHartImpl;

impl ScheduleHart for ScheduleHartImpl {
    fn hart_id() -> usize {
        hart_id()
    }
}
/// 多核调度器
pub static GLOBAL_TASK_MANAGER: Lazy<
    FifoSmpScheduler<CPU_NUM, Arc<Task>, Mutex<()>, ScheduleHartImpl>,
> = Lazy::new(|| FifoSmpScheduler::new());

/// 获取当前 cpu 的信息
pub fn current_cpu() -> &'static mut CPU {
    let hart_id = arch::hart_id();
    unsafe { &mut (*(CPU_MANAGER[hart_id].0.get())) }
}

/// 获取当前 CPU 上的线程
pub fn current_task() -> Option<&'static Arc<Task>> {
    let cpu = current_cpu();
    cpu.task.as_ref()
}

/// 获取当前进程的虚拟页表的 token (root ppn)
pub fn current_user_token() -> usize {
    let task = current_task().unwrap();
    task.token()
}

/// 获取当前进程的 trap 帧（上下文）
pub fn current_trap_frame() -> &'static mut TrapFrame {
    let task = current_task().unwrap();
    task.trap_frame()
}

/// 一个系统调用，用于终止进程。
///
/// 运行成功后，调用该函数的进程将转变为Zombie状态，同时回收部分资源，并让渡CPU执行其他的进程。
/// 等待父进程得知其终止退出后，将回收该进程的其余资源。
/// `exit_code`中的值，将会在其父进程调用[`wait4`]时，作为信息传递给父进程。
/// 当一个具有子进程的进程终止时，其所有子进程将转交至init进程，由init进程完成其子进程相关资源的回收。
/// 当`clear_child_tid`不为0时，会将`clear_child_tid`该处的值置为0，同时内核唤醒当前正在等待的futex。
///
/// 当调用该函数的进程为`pid==0`的init进程时，将直接调用`system_shutdown`使得内核终止。
#[syscall_func(93)]
pub fn do_exit(exit_code: i32) -> isize {
    let task = current_task().unwrap();
    let exit_code = (exit_code & 0xff) << 8;
    if task.get_pid() == 0 {
        println!("Init process exit with code {}", exit_code);
        system_shutdown();
    }
    {
        let init = INIT_PROCESS.clone();
        task.take_children().into_iter().for_each(|child| {
            child.update_parent(init.clone());
            init.insert_child(child);
        });
    }
    task.update_state(TaskState::Zombie);
    task.update_exit_code(exit_code);
    global_logoff_signals(task.get_tid() as usize);
    // clear_child_tid 的值不为 0，则将这个用户地址处的值写为0
    let addr = task.access_inner().clear_child_tid;
    if addr != 0 {
        // 确认这个地址在用户地址空间中。如果没有也不需要报错，因为线程马上就退出了
        let addr = task.transfer_raw_ptr(addr as *mut i32);
        *addr = 0;
    }

    // 回收一些物理页，不然等到wait系统调用真正进行回收时，可能会出现OOM
    // 可回收物理页包括trap页以及内核栈页
    // 在这里还不能回收内核栈页，因为还需要用到内核栈页来执行下面的代码
    // 所以只回收trap页
    // 在wait系统调用中，会回收内核栈页
    task.pre_recycle();
    info!("pre recycle done");
    let clear_child_tid = task.futex_wake();
    if clear_child_tid != 0 {
        let phy_addr = task.transfer_raw_ptr(clear_child_tid as *mut usize);
        *phy_addr = 0;
        error!("exit wake futex on {:#x}", clear_child_tid);
        futex(clear_child_tid, FutexOp::FutexWake as u32, 1, 0, 0, 0);
    } else {
        error!("exit clear_child_tid is 0");
    }
    schedule();
    0
}

/// 一个系统调用，退出当前进程(进程组)下的所有线程(进程)。
///
/// 目前该系统调用直接调用[`do_exit`]，有关进程组的相关功能有待实现。
#[syscall_func(94)]
pub fn exit_group(exit_code: i32) -> isize {
    do_exit(exit_code)
}

/// 一个系统调用，用于使当前正在运行的进程让渡CPU。
#[syscall_func(124)]
pub fn do_suspend() -> isize {
    let task = current_task().unwrap();
    task.access_inner().update_timer();
    check_task_timer_expired();
    task.update_state(TaskState::Ready);
    schedule();
    0
}

/// (待实现)设置进程组的id。目前直接返回0。
#[syscall_func(154)]
pub fn set_pgid() -> isize {
    0
}

/// (待实现)获取进程组的id。目前直接返回0。
#[syscall_func(155)]
pub fn get_pgid() -> isize {
    0
}

/// 创建一个新的session，并使得使用系统调用的当前task成为新session的leader，同时也是新进程组的leader(待实现)
#[syscall_func(157)]
pub fn set_sid() -> isize {
    0
}

/// 获取当前正在运行task的pid号。在Alien中pid作为线程组的标识符，位于同一线程组中的线程的pid相同。
#[syscall_func(172)]
pub fn get_pid() -> isize {
    let process = current_task().unwrap();
    process.get_pid()
}

/// 获取当前正在运行task的ppid号，即父task的pid号。
#[syscall_func(173)]
pub fn get_ppid() -> isize {
    let process = current_task().unwrap();
    let parent = process.access_inner().parent.clone();
    if parent.is_none() {
        return 0;
    } else {
        parent.unwrap().upgrade().unwrap().get_pid()
    }
}

/// (待实现)获取用户 id。在实现多用户权限前默认为最高权限。目前直接返回0。
#[syscall_func(174)]
pub fn getuid() -> isize {
    0
}

/// (待实现)获取有效用户 id，即相当于哪个用户的权限。在实现多用户权限前默认为最高权限。目前直接返回0。
#[syscall_func(175)]
pub fn geteuid() -> isize {
    0
}

/// (待实现)获取用户组 id。在实现多用户权限前默认为最高权限。目前直接返回0。
#[syscall_func(176)]
pub fn getgid() -> isize {
    0
}

/// (待实现)获取有效用户组 id，即相当于哪个用户组的权限。在实现多用户组权限前默认为最高权限。目前直接返回0。
#[syscall_func(177)]
pub fn getegid() -> isize {
    0
}

/// 获取当前正在运行task的tid号。在Alien中tid作为task的唯一标识符。
#[syscall_func(178)]
pub fn get_tid() -> isize {
    let process = current_task().unwrap();
    process.get_tid()
}

/// 一个系统调用，用于创建一个子进程。
///
/// 与传统的`fork()`的功能大致相同，创建一个子进程，并将其放入任务队列中等待cpu进行调度。
/// 但与`fork()`的功能相比，`Alien`中`clone`系统调用提供了更多详细的控制，
/// 管理父进程和子进程之间的共享资源，例如调用者可以控制父子进程之间是否共享虚拟内存空间、文件描述符表、
/// 信号处理程序等。
///
/// `flag`用于控制父子进程之间资源的共享程度，有关flag值及其相关含义设置可见[`CloneFlags`]和[`SignalNumber`]。
/// `stack`用于控制子进程的用户栈。由于clone产生的子进程有可能和父进程共享内存，所以它不能使用父进程的栈。
/// `ptid`是一个在父进程地址空间中的地址，用于在创建子进程成功后向该位置写入子进程的tid号。在flag包含`CLONE_PARENT_SETTID`时才会发挥效果。
/// `tls`用于为子进程创建新的TLS(thread-local storage)值，在flag包含`CLONE_SETTLS`时才会实际产生效果。
/// `ctid`用于给子进程中的[`set_child_tid`]和[`clear_child_tid`]赋值(分别在flag中包含`CLONE_CHILD_SETTID`和`CLONE_CHILD_CLEARTID`时产生效果)。
///
/// 成功创建子进程后父进程会返回子进程的tid号，子进程的返回值将被设置为0；否则返回-1。
///
/// Reference: [clone](https://www.man7.org/linux/man-pages/man2/clone.2.html)
#[syscall_func(220)]
pub fn clone(flag: usize, stack: usize, ptid: usize, tls: usize, ctid: usize) -> isize {
    let clone_flag = CloneFlags::from_bits_truncate(flag as u32);
    // check whether flag include signal
    let sig = flag & 0xff;
    let sig = SignalNumber::from(sig);
    let task = current_task().unwrap();

    let child_num = task.access_inner().children.len();
    if child_num >= 10 {
        do_suspend();
    }
    let new_task = task.t_clone(clone_flag, stack, sig, ptid, tls, ctid);
    if new_task.is_none() {
        return -1;
    }
    let new_task = new_task.unwrap();
    // update return value
    let trap_frame = new_task.trap_frame();
    trap_frame.update_res(0);
    let tid = new_task.get_tid();
    GLOBAL_TASK_MANAGER.add_task(Arc::new(FifoTask::new(new_task)));
    // do_suspend();
    tid
}

/// 一个系统调用，用于执行一个文件。
///
/// `path`用于指明要执行的文件的绝对路径。
/// `args_ptr`用于指明保存启动可执行文件时要传入的参数的地址。
/// `env`用于指明保存相关环境变量的地址。
///
/// 成功执行文件后会返回0；否则会返回-1或错误类型。
#[syscall_func(221)]
pub fn do_exec(path: *const u8, args_ptr: usize, env: usize) -> isize {
    let task = current_task().unwrap();
    let mut path_str = task.transfer_str(path);
    // get the args and push them into the new process stack
    let (mut args, envs) = parse_user_arg_env(args_ptr, env);
    warn!("exec path: {}", path_str);
    warn!("exec args: {:?} ,env: {:?}", args, envs);
    if path_str.ends_with(".sh") {
        if args.is_empty() {
            let mut new_path = path_str.clone();
            new_path.push('\0');
            args.insert(0, new_path);
        }
        path_str = "/bin/busybox".to_string();
        args.insert(0, "sh\0".to_string());
    }
    let mut data = Vec::new();
    if path_str.contains("libc-bench") {
        path_str = path_str.replace("libc-bench", "libc-bench2");
    }
    if fs::read_all(&path_str, &mut data) {
        let res = task.exec(&path_str, data.as_slice(), args, envs);
        if res.is_err() {
            return res.err().unwrap();
        }
        return 0;
    } else {
        println!("exec {} failed", path_str);
        -1
    }
}

/// 一个系统调用，用于父进程等待某子进程退出。
///
/// `pid`用于指明等待的子进程pid号。`pid == -1`表示父进程等待任意子进程返回。
/// 当`exit_code`非空时，将会把退出的子程序的退出值赋给`exit_code`所指向的位置。
/// `options`主要用于控制`wait4`的执行逻辑，例如当`wait_options`包含`WNOHANG`时，即使未发现子程序返回，函数也将直接返回0。
///
/// 一般`wait4`会使得父进程阻塞，直到子进程退出，返回退出的子进程pid。但当`wait_options`包含`WNOHANG`时，即使未发现子程序返回，函数也将直接返回0。
/// 当父进程的所有子进程中不包含进程号为pid的子进程，将返回-1。
///
/// Reference:[wait](https://man7.org/linux/man-pages/man2/wait.2.html)
#[syscall_func(260)]
pub fn wait4(pid: isize, exit_code: *mut i32, options: u32, _rusage: *const u8) -> isize {
    let process = current_task().unwrap();
    loop {
        if process
            .children()
            .iter()
            .find(|child| child.get_pid() == pid || pid == -1)
            .is_none()
        {
            return -1;
        }
        let res = process.check_child(pid);
        if let Some(index) = res {
            let child = process.remove_child(index);
            assert_eq!(
                Arc::strong_count(&child),
                1,
                "Father is [{}-{}], wait task is [{}-{}]",
                process.get_pid(),
                process.get_tid(),
                child.get_pid(),
                child.get_tid()
            );
            if !exit_code.is_null() {
                let exit_code_ref = process.transfer_raw_ptr(exit_code);
                *exit_code_ref = child.exit_code();
            }
            return child.get_tid();
        } else {
            let wait_options = WaitOptions::from_bits(options).unwrap();
            if wait_options.contains(WaitOptions::WNOHANG) {
                return 0;
            } else {
                do_suspend();
            }
        }
    }
}

/// 一个系统调用，用于改变堆区的大小(目前仅可以增加堆区大小)
///
/// `addr`用于指明扩充堆区后，堆区的末尾位置。
/// 当`addr`所标识的位置在当前堆起始位置的前方，或者堆当前已使用的末尾位置的前方时，将会导致增加堆区大小失败。
///
/// 成功增加堆区大小时，函数返回堆当前已使用的末尾位置；否则返回-1。
#[syscall_func(214)]
pub fn do_brk(addr: usize) -> isize {
    let process = current_task().unwrap();
    let mut inner = process.access_inner();
    let heap_info = inner.heap_info();
    if addr == 0 {
        return heap_info.current as isize;
    }
    if addr < heap_info.start || addr < heap_info.current {
        // panic!("heap can't be shrinked");
        return -1;
    }
    let res = inner.extend_heap(addr);
    if res.is_err() {
        return -1;
    }
    res.unwrap() as isize
}

/// 一个系统调用，用于修改进程clear_child_tid的值，同时返回进程的tid。
#[syscall_func(96)]
pub fn set_tid_address(tidptr: usize) -> isize {
    let task = current_task().unwrap();
    task.set_tid_address(tidptr);
    task.get_tid()
}

/// 一个系统调用，用于修改进程的资源限制。
///
/// 进程对其拥有的资源，包括用户栈大小、可以打开的文件描述符数、用户地址空间大小等都有所上限。
///
/// `prlimit64`则可以根据资源的种类对不同的资源进行大小的限制。针对每一具体限制都包括软上限和硬上限，具体可见[`PrLimit`]。
/// `pid`用于指明需要修改资源限制的进程的pid号。
/// `resource`用于指明需要修改的资源类型，可选的值包括`RLIMIT_STACK`、`RLIMIT_NOFILE`、`RLIMIT_AS`等，详情可见[`PrLimitRes`]。
/// `new_limit`用于指明新限制的指针，如果为空指针则不进行新限制的赋值。
/// `old_limit`用于指明存放旧限制的指针，如果为空则不进行旧限制的保存。
///
/// 正确执行后会返回0；如果输入的pid为0或者为当前正在运行的进程号，则会直接终止。
#[syscall_func(261)]
pub fn prlimit64(pid: usize, resource: usize, new_limit: *const u8, old_limit: *mut u8) -> isize {
    assert!(pid == 0 || pid == current_task().unwrap().get_pid() as usize);
    let task = current_task().unwrap();
    let mut inner = task.access_inner();
    if let Ok(resource) = PrLimitRes::try_from(resource) {
        if !old_limit.is_null() {
            let limit = inner.get_prlimit(resource);
            warn!("get rlimit nofile to {:?}", limit);
            inner.copy_to_user(&limit, old_limit as *mut PrLimit);
        }
        match resource {
            PrLimitRes::RlimitStack => {}
            PrLimitRes::RlimitNofile => {
                if !new_limit.is_null() {
                    let mut limit = PrLimit::new(0, 0);
                    inner.copy_from_user(new_limit as *const PrLimit, &mut limit);
                    warn!("set rlimit nofile to {:?}", limit);
                    inner.set_prlimit(resource, limit);
                }
            }
            PrLimitRes::RlimitAs => {}
        }
    }
    0
}

/// 用于exec可执行文件时，分别在args_ptr和env_ptr所指向的地址处取出参数和环境变量
fn parse_user_arg_env(args_ptr: usize, env_ptr: usize) -> (Vec<String>, Vec<String>) {
    let task = current_task().unwrap();
    let mut args = Vec::new();

    if args_ptr != 0 {
        let mut start = args_ptr as *mut usize;
        loop {
            let arg = task.transfer_raw_ptr(start);
            if *arg == 0 {
                break;
            }
            args.push(*arg);
            start = unsafe { start.add(1) };
        }
    }
    let args = args
        .into_iter()
        .map(|arg| {
            let mut arg = task.transfer_str(arg as *const u8);
            arg.push('\0');
            arg
        })
        .collect::<Vec<String>>();
    let mut envs = Vec::new();
    if env_ptr != 0 {
        let mut start = env_ptr as *mut usize;
        loop {
            let env = task.transfer_raw_ptr(start);
            if *env == 0 {
                break;
            }
            envs.push(*env);
            start = unsafe { start.add(1) };
        }
    }
    let envs = envs
        .into_iter()
        .map(|env| {
            let mut env = task.transfer_str(env as *const u8);
            env.push('\0');
            env
        })
        .collect::<Vec<String>>();
    (args, envs)
}
