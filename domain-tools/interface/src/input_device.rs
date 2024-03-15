use crate::DeviceBase;
use rref::RpcResult;

pub trait InputDomain: DeviceBase {
    /// Read an input event from the input device
    fn event(&self) -> RpcResult<Option<u64>>;
}
