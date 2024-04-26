use alloc::{
    string::{String, ToString},
    vec::Vec,
};

use constants::{AlienError, AlienResult};
use memory_addr::VirtAddr;

use crate::processor::current_task;

pub fn do_execve(
    filename_ptr: VirtAddr,
    argv_ptr: VirtAddr,
    envp_ptr: VirtAddr,
) -> AlienResult<isize> {
    let task = current_task().unwrap();
    let mut path_str = task.read_string_from_user(filename_ptr)?;
    // get the args and push them into the new process stack
    let (mut args, envs) = parse_user_argv_envp(argv_ptr, envp_ptr);
    warn!("exec path: {}", path_str);
    warn!("exec args: {:?} ,env: {:?}", args, envs);
    if path_str.ends_with(".sh") {
        if args.is_empty() {
            let mut new_path = path_str.clone();
            new_path.push('\0');
            args.insert(0, new_path);
        }
        path_str = "./busybox".to_string();
        args.insert(0, "sh\0".to_string());
    }
    let mut data = Vec::new();
    if crate::vfs_shim::read_all(&path_str, &mut data) {
        let res = task.do_execve(&path_str, data.as_slice(), args, envs);
        if res.is_err() {
            return Err(AlienError::ENOEXEC);
        }
        info!("exec {} success", path_str);
        Ok(0)
    } else {
        warn!("exec {} failed", path_str);
        Err(AlienError::ENOENT)
    }
}

fn parse_user_argv_envp(argv_ptr: VirtAddr, envp_ptr: VirtAddr) -> (Vec<String>, Vec<String>) {
    let task = current_task().unwrap();
    let mut argv = Vec::new();
    if argv_ptr != VirtAddr::from(0) {
        let mut start = argv_ptr;
        loop {
            let arg_ptr = task.read_val_from_user(start).unwrap();
            if arg_ptr == 0 {
                break;
            }
            argv.push(arg_ptr);
            start += core::mem::size_of::<usize>();
        }
    }
    let argv = argv
        .into_iter()
        .map(|arg_ptr| {
            let mut arg = task.read_string_from_user(arg_ptr).unwrap();
            arg.push('\0');
            arg
        })
        .collect::<Vec<String>>();
    let mut envp = Vec::new();
    if envp_ptr != VirtAddr::from(0) {
        let mut start = envp_ptr;
        loop {
            let env = task.read_val_from_user(start).unwrap();
            if env == 0 {
                break;
            }
            envp.push(env);
            start += core::mem::size_of::<usize>();
        }
    }
    let envp = envp
        .into_iter()
        .map(|env| {
            let mut env = task.read_string_from_user(env).unwrap();
            env.push('\0');
            env
        })
        .collect::<Vec<String>>();
    (argv, envp)
}
