use crate::DeviceBase;
use rref::{RRef, RpcResult};

pub trait BlkDeviceDomain: DeviceBase {
    fn read_block(&self, block: u32, data: RRef<[u8; 512]>) -> RpcResult<RRef<[u8; 512]>>;
    fn write_block(&self, block: u32, data: &RRef<[u8; 512]>) -> RpcResult<usize>;
    fn get_capacity(&self) -> RpcResult<u64>;
    fn flush(&self) -> RpcResult<()>;
    fn restart(&self) -> bool {
        false
    }
}
