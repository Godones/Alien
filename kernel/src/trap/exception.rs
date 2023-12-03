//! Alien 中对于内部异常的处理
//!
//! 目前包括系统调用异常处理 [`syscall_exception_handler`]、页错误异常处理 [`page_exception_handler`] (包括
//! 指令页错误异常处理 [`instruction_page_fault_exception_handler`]、 加载页错误异常处理[`load_page_fault_exception_handler`]、
//! 储存页错误异常处理 [`store_page_fault_exception_handler`]) 和 文件读入异常处理 [`trap_common_read_file`]。
use crate::arch::{interrupt_enable};
use crate::error::{AlienError, AlienResult};
use crate::fs::file::File;
use crate::task::{current_task, current_trap_frame};
use alloc::sync::Arc;
use riscv::register::scause::{Exception, Trap};

/// 系统调用异常处理
pub fn syscall_exception_handler() {
    // enable interrupt
    interrupt_enable();
    // jump to next instruction anyway
    let mut cx = current_trap_frame();
    cx.update_sepc();
    // get system call return value
    let parameters = cx.parameters();
    let syscall_name = pconst::syscall_name(parameters[0]);

    let p_name = current_task().unwrap().get_name();
    let tid = current_task().unwrap().get_tid();
    let pid = current_task().unwrap().get_pid();
    if !p_name.contains("shell") && !p_name.contains("init") && !p_name.contains("ls") {
        // ignore shell and init
        info!(
            "[pid:{}, tid: {}][p_name: {}] syscall: [{}] {}({:#x}, {:#x}, {:#x}, {:#x}, {:#x}, {:#x})",
            pid,
            tid,
            p_name,
            parameters[0],
            syscall_name,
            parameters[1],
            parameters[2],
            parameters[3],
            parameters[4],
            parameters[5],
            parameters[6]
        );
    }

    let result = invoke_call_id!(
        parameters[0],
        parameters[1],
        parameters[2],
        parameters[3],
        parameters[4],
        parameters[5],
        parameters[6]
    );
    let result = Some(result);
    // cx is changed during sys_exec, so we have to call it again
    cx = current_trap_frame();

    if !p_name.contains("shell") && !p_name.contains("init") && !p_name.contains("ls") {
        info!(
            "[pid:{}, tid: {}] syscall: [{}] result: {:?}, tp: {:#x}",
            pid,
            tid,
            syscall_name,
            result,
            cx.regs()[4]
        );
    }
    cx.update_res(result.unwrap() as usize);
}

/// 页异常处理，会根据不同的异常类型，分发至指令页错误异常处理 [`instruction_page_fault_exception_handler`]、
/// 加载页错误异常处理 [`load_page_fault_exception_handler`]、
/// 储存页错误异常处理 [`store_page_fault_exception_handler`] 等处理方案中。
pub fn page_exception_handler(trap: Trap, addr: usize) -> AlienResult<()> {
    trace!(
        "[pid: {}] page fault addr:{:#x} trap:{:?}",
        current_task().unwrap().get_pid(),
        addr,
        trap
    );
    match trap {
        Trap::Exception(Exception::LoadPageFault) => load_page_fault_exception_handler(addr)?,
        Trap::Exception(Exception::StorePageFault) => store_page_fault_exception_handler(addr)?,
        Trap::Exception(Exception::InstructionPageFault) => {
            instruction_page_fault_exception_handler(addr)?
        }
        _ => {
            return Err(AlienError::ENOSYS);
        }
    }
    Ok(())
}

/// 指令页错误异常处理
pub fn instruction_page_fault_exception_handler(addr: usize) -> AlienResult<()> {
    let task = current_task().unwrap();
    trace!(
        "[tid: {}] do instruction_page_fault  addr:{:#x}",
        task.get_tid(),
        addr
    );
    let res = task.access_inner().do_instruction_page_fault(addr)?;
    if res.is_some() {
        let (file, buf, offset) = res.unwrap();
        if file.is_some() {
            trap_common_read_file(file.unwrap(), buf, offset);
        }
    }
    Ok(())
}

/// 加载页错误异常处理
pub fn load_page_fault_exception_handler(addr: usize) -> AlienResult<()> {
    let info = {
        let process = current_task().unwrap();
        process.access_inner().do_load_page_fault(addr)?
    };
    if info.is_some() {
        let (file, buf, offset) = info.unwrap();
        if file.is_some() {
            trap_common_read_file(file.unwrap(), buf, offset);
        }
    }
    Ok(())
}

/// 储存页错误异常处理
pub fn store_page_fault_exception_handler(addr: usize) -> AlienResult<()> {
    let process = current_task().unwrap();
    trace!(
        "[tid: {}] do store page fault addr:{:#x}",
        process.get_tid(),
        addr
    );
    let res = process.access_inner().do_store_page_fault(addr)?;
    if res.is_some() {
        let (file, buf, offset) = res.unwrap();
        if file.is_some() {
            trap_common_read_file(file.unwrap(), buf, offset);
        }
    }
    Ok(())
}

/// 文件读入异常处理
pub fn trap_common_read_file(file: Arc<dyn File>, buf: &mut [u8], offset: u64) {
    error!(
        "trap_common_read_file buf.len: {}, offset:{:#x}",
        buf.len(),
        offset
    );
    // let r = vfs_read_file::<VfsProvider>(file.get_file(), buf, offset);
    let r = file.read_at(offset, buf);
    if r.is_err() {
        error!("page fault: read file error");
    }
}
