use alloc::string::String;

use constants::{AlienResult, LinuxErrno};
use ksync::Mutex;

type FD = usize;
#[allow(unused)]
pub struct UnixSocket {
    /// 文件路径，即套接字地址
    file_path: Mutex<Option<String>>,
    /// 套接字数据
    file: Mutex<Option<FD>>,
}

impl UnixSocket {
    /// 创建一个新的 Unix 协议族下的套接字结构
    pub fn new() -> Self {
        Self {
            file_path: Mutex::new(None),
            file: Mutex::new(None),
        }
    }

    /// UnixSocket 的 connect 操作
    pub fn connect(&self, _file_path: String) -> AlienResult<()> {
        Err(LinuxErrno::ENOENT)
    }
}
