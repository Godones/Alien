use riscv::register::scause::{Exception, Trap};
use rvfs::file::vfs_read_file;

use crate::arch::interrupt_enable;
use crate::fs::vfs::VfsProvider;
use crate::syscall;
use crate::task::{current_process, current_trap_frame, do_exit};

pub fn syscall_exception_handler() {
    // enable interrupt
    interrupt_enable();
    // jump to next instruction anyway
    let mut cx = current_trap_frame();
    cx.update_sepc();
    // get system call return value
    let parameters = cx.parameters();
    let result = syscall::do_syscall(parameters[0], &parameters[1..]);
    // cx is changed during sys_exec, so we have to call it again
    cx = current_trap_frame();
    cx.update_res(result as usize);
}

/// the solution for page fault
pub fn page_exception_handler(trap: Trap, addr: usize) {
    match trap {
        Trap::Exception(Exception::LoadPageFault) => {
            load_page_fault_exception_handler(addr)
        }
        _ => {
            do_exit(-1);
        }
    }
}


pub fn load_page_fault_exception_handler(addr: usize) {
    let info = {
        let process = current_process().unwrap();
        process.access_inner().do_load_page_fault(addr)
    };
    if info.is_err() {
        do_exit(-1);
    }
    let (file, buf, offset) = info.unwrap();
    let r = vfs_read_file::<VfsProvider>(file, buf, offset);
    println!("read file result: {:?}", r);
}

/// the solution for illegal instruction
pub fn illegal_instruction_exception_handler() {
    do_exit(-3);
}
