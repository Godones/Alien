use core::error::Error;
use core::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum AlienError {
    NoSpace,
    Other,
    InvalidSyscall(usize),
}

pub type AlienResult<T> = Result<T, AlienError>;

impl Display for AlienError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            AlienError::NoSpace => write!(f, "No space"),
            AlienError::Other => write!(f, "Other error"),
            AlienError::InvalidSyscall(num) => write!(f, "Invalid syscall number: {}", num),
        }
    }
}

impl Error for AlienError {}
