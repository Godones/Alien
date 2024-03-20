use crate::{DeviceBase, DeviceInfo};
use constants::AlienResult;

pub trait UartDomain: DeviceBase {
    fn init(&self, device_info: &DeviceInfo) -> AlienResult<()>;
    /// Write a character to the UART
    fn putc(&self, ch: u8) -> AlienResult<()>;
    /// Read a character from the UART
    fn getc(&self) -> AlienResult<Option<u8>>;
    /// Check if there is data to get from the UART
    fn have_data_to_get(&self) -> AlienResult<bool>;
    /// Check if there is space to put data to the UART
    fn have_space_to_put(&self) -> AlienResult<bool> {
        Ok(true)
    }
}
