use alloc::sync::Arc;

use spin::Once;

use crate::interrupt::DeviceBase;

pub trait UartDevice: Send + Sync + DeviceBase {
    fn put(&self, c: u8);
    fn get(&self) -> Option<u8>;
    fn put_bytes(&self, bytes: &[u8]);
    fn have_data_to_get(&self) -> bool;
    fn have_space_to_put(&self) -> bool;
}

pub static UART_DEVICE: Once<Arc<dyn UartDevice>> = Once::new();

pub fn init_uart(uart: Arc<dyn UartDevice>) {
    UART_DEVICE.call_once(|| uart);
}
