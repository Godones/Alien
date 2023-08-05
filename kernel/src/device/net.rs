use alloc::sync::Arc;

use spin::Once;

use crate::interrupt::DeviceBase;

pub trait NetDevice: DeviceBase {}


pub static NET_DEVICE: Once<Arc<dyn NetDevice>> = Once::new();


pub fn init_net_device(net: Arc<dyn NetDevice>) {
    NET_DEVICE.call_once(|| net);
}