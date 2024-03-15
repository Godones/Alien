use crate::Basic;
use rref::RpcResult;

pub trait SysCallDomain: Basic {
    fn call(&self, syscall_id: usize, args: [usize; 6]) -> RpcResult<isize>;
}
