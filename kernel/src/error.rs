use pconst::LinuxErrno;

pub type AlienError = LinuxErrno;
pub type AlienResult<T> = Result<T, AlienError>;
