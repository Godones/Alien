use crate::syscall::{
    sys_create_global_bucket, sys_dbfs_execute_operate, sys_execute_user_func, sys_show_dbfs,
};
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
pub use dbop::*;
pub use jammdb::Bucket;

pub fn dbfs_execute_operate(bucket: &str, operate: OperateSet) -> isize {
    let mut operate = serde_json::to_string(&operate).unwrap();
    operate.push('\0');
    if !bucket.ends_with('\0') {
        let mut bucket = bucket.to_string();
        bucket.push('\0');
        return sys_dbfs_execute_operate(bucket.as_ptr(), operate.as_ptr());
    }
    sys_dbfs_execute_operate(bucket.as_ptr(), operate.as_ptr())
}

pub fn create_global_bucket(key: &str) -> isize {
    if !key.ends_with('\0') {
        let mut key = key.to_string();
        key.push('\0');
        return sys_create_global_bucket(key.as_ptr());
    }
    sys_create_global_bucket(key.as_ptr())
}

pub fn execute_user_func(key: &str, func: *const (), buf: &mut [u8]) -> isize {
    if !key.ends_with('\0') {
        let mut key = key.to_string();
        key.push('\0');
        return sys_execute_user_func(key.as_ptr(), buf.as_ptr(), buf.len(), func as usize);
    }
    sys_execute_user_func(key.as_ptr(), buf.as_ptr(), buf.len(), func as usize)
}

pub fn show_dbfs() -> isize {
    sys_show_dbfs()
}
