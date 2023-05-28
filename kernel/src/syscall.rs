use spin::Once;
use syscall_table::{register_syscall, Table};
static SYSCALL_TABLE: Once<Table> = Once::new();
pub fn register_all_syscall(){
	let mut table = Table::new();
	register_syscall!(table,
	(160, sys_uname),
	(210, shutdown),
	(169, get_time_of_day),
	(153, times),
	(101, sys_nanosleep),
	(59, sys_pipe),
	(23, sys_dup),
	(24, sys_dup2),
	(93, do_exit),
	(124, do_suspend),
	(172, get_pid),
	(173, get_ppid),
	(220, clone),
	(221, do_exec),
	(260, wait_pid),
	(214, do_brk),
	(40, sys_mount),
	(39, sys_umount),
	(56, sys_openat),
	(57, sys_close),
	(61, sys_getdents),
	(45, sys_truncate),
	(46, sys_ftruncate),
	(63, sys_read),
	(64, sys_write),
	(17, sys_getcwd),
	(49, sys_chdir),
	(83, sys_mkdir),
	(1000, sys_list),
	(62, sys_lseek),
	(80, sys_fstat),
	(37, sys_linkat),
	(35, sys_unlinkat),
	(36, sys_symlinkat),
	(78, sys_readlinkat),
	(79, sys_fstateat),
	(44, sys_fstatfs),
	(43, sys_statfs),
	(38, sys_renameat),
	(34, sys_mkdirat),
	(5, sys_setxattr),
	(6, sys_lsetxattr),
	(7, sys_fsetxattr),
	(8, sys_getxattr),
	(9, sys_lgetxattr),
	(10, sys_fgetxattr),
	(11, sys_listxattr),
	(12, sys_llistxattr),
	(13, sys_flistxattr),
	(14, sys_removexattr),
	(15, sys_lremovexattr),
	(16, sys_fremovexattr),
	(215, do_munmap),
	(222, do_mmap),

	);
	SYSCALL_TABLE.call_once(||table);
}
pub fn do_syscall(id:usize,args:&[usize])->isize{
	let res = SYSCALL_TABLE.get().unwrap().do_call(id,&args);
	if res.is_none(){
		    return -1;
	}else {
	    return res.unwrap();
	}
}
use crate::fs::sys_chdir;
use crate::fs::sys_close;
use crate::fs::sys_fgetxattr;
use crate::fs::sys_flistxattr;
use crate::fs::sys_fremovexattr;
use crate::fs::sys_fsetxattr;
use crate::fs::sys_fstat;
use crate::fs::sys_fstateat;
use crate::fs::sys_fstatfs;
use crate::fs::sys_ftruncate;
use crate::fs::sys_getcwd;
use crate::fs::sys_getdents;
use crate::fs::sys_getxattr;
use crate::fs::sys_lgetxattr;
use crate::fs::sys_linkat;
use crate::fs::sys_list;
use crate::fs::sys_listxattr;
use crate::fs::sys_llistxattr;
use crate::fs::sys_lremovexattr;
use crate::fs::sys_lseek;
use crate::fs::sys_lsetxattr;
use crate::fs::sys_mkdir;
use crate::fs::sys_mkdirat;
use crate::fs::sys_mount;
use crate::fs::sys_openat;
use crate::fs::sys_read;
use crate::fs::sys_readlinkat;
use crate::fs::sys_removexattr;
use crate::fs::sys_renameat;
use crate::fs::sys_setxattr;
use crate::fs::sys_statfs;
use crate::fs::sys_symlinkat;
use crate::fs::sys_truncate;
use crate::fs::sys_umount;
use crate::fs::sys_unlinkat;
use crate::fs::sys_write;
use crate::ipc::sys_dup;
use crate::ipc::sys_dup2;
use crate::ipc::sys_pipe;
use crate::memory::do_mmap;
use crate::memory::do_munmap;
use crate::sbi::shutdown;
use crate::system::sys_uname;
use crate::task::clone;
use crate::task::do_brk;
use crate::task::do_exec;
use crate::task::do_exit;
use crate::task::do_suspend;
use crate::task::get_pid;
use crate::task::get_ppid;
use crate::task::wait_pid;
use crate::timer::get_time_of_day;
use crate::timer::sys_nanosleep;
use crate::timer::times;
