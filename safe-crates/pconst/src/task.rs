use bitflags::bitflags;

/*
/*
 * cloning flags:
 */
#define CSIGNAL		0x000000ff	/* signal mask to be sent at exit */
#define CLONE_VM	0x00000100	/* set if VM shared between processes */
#define CLONE_FS	0x00000200	/* set if fs info shared between processes */
#define CLONE_FILES	0x00000400	/* set if open files shared between processes */
#define CLONE_SIGHAND	0x00000800	/* set if signal handlers and blocked signals shared */
#define CLONE_PIDFD	0x00001000	/* set if a pidfd should be placed in parent */
#define CLONE_PTRACE	0x00002000	/* set if we want to let tracing continue on the child too */
#define CLONE_VFORK	0x00004000	/* set if the parent wants the child to wake it up on mm_release */
#define CLONE_PARENT	0x00008000	/* set if we want to have the same parent as the cloner */
#define CLONE_THREAD	0x00010000	/* Same thread group? */
#define CLONE_NEWNS	0x00020000	/* New mount namespace group */
#define CLONE_SYSVSEM	0x00040000	/* share system V SEM_UNDO semantics */
#define CLONE_SETTLS	0x00080000	/* create a new TLS for the child */
#define CLONE_PARENT_SETTID	0x00100000	/* set the TID in the parent */
#define CLONE_CHILD_CLEARTID	0x00200000	/* clear the TID in the child */
#define CLONE_DETACHED		0x00400000	/* Unused, ignored */
#define CLONE_UNTRACED		0x00800000	/* set if the tracing process can't force CLONE_PTRACE on this clone */
#define CLONE_CHILD_SETTID	0x01000000	/* set the TID in the child */
#define CLONE_NEWCGROUP		0x02000000	/* New cgroup namespace */
#define CLONE_NEWUTS		0x04000000	/* New utsname namespace */
#define CLONE_NEWIPC		0x08000000	/* New ipc namespace */
#define CLONE_NEWUSER		0x10000000	/* New user namespace */
#define CLONE_NEWPID		0x20000000	/* New pid namespace */
#define CLONE_NEWNET		0x40000000	/* New network namespace */
#define CLONE_IO		0x80000000	/* Clone io context */

/* Flags for the clone3() pconst. */
#define CLONE_CLEAR_SIGHAND 0x100000000ULL /* Clear any signal handler and reset to SIG_DFL. */
#define CLONE_INTO_CGROUP 0x200000000ULL /* Clone into a specific cgroup given the right permissions. */

/*
 * cloning flags intersect with CSIGNAL so can be used with unshare and clone3
 * syscalls only:
 */
#define CLONE_NEWTIME	0x00000080	/* New time namespace */

*/

bitflags! {
    pub struct CloneFlags: u32 {
        const CLONE_NEWTIME = 0x00000080;
        /// 共享地址空间
        const CLONE_VM = 0x00000100;
        /// 共享文件系统新信息
        const CLONE_FS = 0x00000200;
        /// 共享文件描述符(fd)表
        const CLONE_FILES = 0x00000400;
        /// 共享信号处理函数
        const CLONE_SIGHAND = 0x00000800;
        /// 创建指向子任务的fd，用于 sys_pidfd_open
        const CLONE_PIDFD = 0x00001000;
        /// 用于 sys_ptrace
        const CLONE_PTRACE = 0x00002000;
        /// 指定父任务创建后立即阻塞，直到子任务退出才继续
        const CLONE_VFORK = 0x00004000;
        /// 指定子任务的 ppid 为当前任务的 ppid，相当于创建“兄弟”而不是“子女”
        const CLONE_PARENT = 0x00008000;
        /// 作为一个“线程”被创建。具体来说，它同 CLONE_PARENT 一样设置 ppid，且不可被 wait
        const CLONE_THREAD = 0x00010000;
        /// 子任务使用新的命名空间。目前还未用到
        const CLONE_NEWNS = 0x00020000;
        /// 子任务共享同一组信号量。用于 sys_semop
        const CLONE_SYSVSEM = 0x00040000;
        /// 要求设置 tls
        const CLONE_SETTLS = 0x00080000;
        /// 要求在父任务的一个地址写入子任务的 tid
        const CLONE_PARENT_SETTID = 0x00100000;
        /// 要求将子任务的一个地址清零。这个地址会被记录下来，当子任务退出时会触发此处的 futex
        const CLONE_CHILD_CLEARTID = 0x00200000;
        /// 历史遗留的 flag，现在按 linux 要求应忽略
        const CLONE_DETACHED = 0x00400000;
        /// 与 sys_ptrace 相关，目前未用到
        const CLONE_UNTRACED = 0x00800000;
        /// 要求在子任务的一个地址写入子任务的 tid
        const CLONE_CHILD_SETTID = 0x01000000;
        const CLONE_NEWCGROUP = 0x02000000;
        const CLONE_NEWUTS = 0x04000000;
        const CLONE_NEWIPC = 0x08000000;
        const CLONE_NEWUSER = 0x10000000;
        const CLONE_NEWPID = 0x20000000;
        const CLONE_NEWNET = 0x40000000;
        const CLONE_IO = 0x80000000;
    }
}

bitflags! {
    pub struct WaitOptions:u32 {
        const WNOHANG = 1;
        const WUNTRACED = 2;
        const WEXITED = 0x4;
        const WCONTINUED = 8;
        const WNOWAIT = 0x01000000;
    }
}
