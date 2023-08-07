use core::error::Error;
use core::fmt::{Display, Formatter};

#[derive(Debug, PartialOrd, PartialEq)]
pub enum AlienError {
    NoSpace,
    Other,
    InvalidSyscall(usize),
    ThreadNeedWait,
    ThreadNeedExit,
}

pub type AlienResult<T> = Result<T, AlienError>;

impl Display for AlienError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            AlienError::NoSpace => write!(f, "No space"),
            AlienError::Other => write!(f, "Other error"),
            AlienError::InvalidSyscall(num) => write!(f, "Invalid syscall number: {}", num),
            AlienError::ThreadNeedWait => write!(f, "ThreadNeedWait"),
            AlienError::ThreadNeedExit => write!(f, "ThreadNeedExit"),
        }
    }
}

impl Error for AlienError {}

/// Return the target if the value is an error.
/// Unwrap the value if the value is not an error.
#[macro_export]
macro_rules! error_unwrap {
    ($value:ident,$target:expr) => {
        if $value.is_err() {
            return $target;
        }
        let $value = $value.unwrap();
    };
}

/// Return the target if the value is None.
/// Unwrap the value if the value is not None.
#[macro_export]
macro_rules! option_unwrap {
    ($value:ident,$target:expr) => {
        if $value.is_none() {
            return $target;
        }
        let $value = $value.unwrap();
    };
}
