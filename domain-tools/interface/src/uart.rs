use crate::DeviceBase;
use rref::RpcResult;

pub trait UartDomain: DeviceBase {
    /// Write a character to the UART
    fn putc(&self, ch: u8) -> RpcResult<()>;
    /// Read a character from the UART
    fn getc(&self) -> RpcResult<Option<u8>>;
    /// Check if there is data to get from the UART
    fn have_data_to_get(&self) -> RpcResult<bool>;
    /// Check if there is space to put data to the UART
    fn have_space_to_put(&self) -> RpcResult<bool> {
        Ok(true)
    }
}
