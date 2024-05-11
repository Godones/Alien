use lose_net_stack::{net_trait::NetInterface, MacAddress};
use rref::RRefVec;

use crate::NET_INTERFACE;

#[derive(Debug)]
pub struct NetMod;

impl NetInterface for NetMod {
    fn send(buf: &[u8]) {
        log::error!("send data {} bytes", buf.len());
        // hexdump(buf);
        let shared_buf = RRefVec::from_slice(buf);
        NET_INTERFACE.get().unwrap().transmit(&shared_buf).unwrap();
    }

    fn local_mac_address() -> MacAddress {
        MacAddress::new([0x52, 0x54, 0x00, 0x12, 0x34, 0x56])
    }
}
