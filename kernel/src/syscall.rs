use spin::Once;
use syscall_table::{register_syscall, Table};
static SYSCALL_TABLE: Once<Table> = Once::new();
pub fn register_all_syscall(){
	let mut table = Table::new();
	register_syscall!(table,
	(1001, sys_create_global_bucket),
	(1002, sys_execute_user_func),
	(1003, sys_show_dbfs),
	(1004, sys_execute_user_operate),
	(56, sys_open),
	(57, sys_close),
	(63, sys_read),
	(64, sys_write),
	(17, sys_getcwd),
	(49, sys_chdir),
	(83, sys_mkdir),
	(1000, sys_list),
	(62, sys_lseek),
	(169, get_time_ms),
	(1005, sleep),
	(210, shutdown),
	(93, do_exit),
	(124, do_suspend),
	(172, get_pid),
	(220, do_fork),
	(221, do_exec),
	(260, wait_pid),

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
use crate::task::do_fork;
use crate::fs::sys_show_dbfs;
use crate::fs::sys_mkdir;
use crate::fs::sys_read;
use crate::fs::sys_getcwd;
use crate::timer::get_time_ms;
use crate::fs::sys_create_global_bucket;
use crate::fs::sys_list;
use crate::fs::sys_chdir;
use crate::sbi::shutdown;
use crate::fs::sys_open;
use crate::fs::sys_close;
use crate::task::do_exec;
use crate::task::wait_pid;
use crate::task::do_suspend;
use crate::task::get_pid;
use crate::timer::sleep;
use crate::fs::sys_write;
use crate::fs::sys_execute_user_func;
use crate::fs::sys_execute_user_operate;
use crate::task::do_exit;
use crate::fs::sys_lseek;
