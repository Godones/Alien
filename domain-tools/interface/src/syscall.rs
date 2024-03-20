use crate::Basic;
use constants::AlienResult;

pub trait SysCallDomain: Basic {
    fn init(&self) -> AlienResult<()>;
    fn call(&self, syscall_id: usize, args: [usize; 6]) -> AlienResult<isize>;
}
