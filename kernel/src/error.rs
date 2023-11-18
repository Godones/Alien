//! alien os中error定义
//!
//! 后续需要将其与linux的error等价转换
use pconst::LinuxErrno;

pub type AlienError = LinuxErrno;
pub type AlienResult<T> = Result<T, AlienError>;
