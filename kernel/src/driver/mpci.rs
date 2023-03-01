use crate::driver::hal::PortImpl;
use fdt::node::FdtNode;
use fdt::standard_nodes::Compatible;
use pci::BAR;
use pci::{scan_bus, CSpaceAccessMethod};
pub fn pci_probe(node: FdtNode) {
    if let Some(reg) = node.reg().and_then(|mut reg| reg.next()) {
        let paddr = reg.starting_address as usize;
        let size = reg.size.unwrap();
        info!("walk dt addr={:#x}, size={:#x}", paddr, size);
        info!(
            "Device tree node {}: {:?}",
            node.name,
            node.compatible().map(Compatible::first),
        );
        unsafe {
            scan_bus(
                &PortImpl,
                CSpaceAccessMethod::MemoryMapped(paddr as *mut u8),
            )
            .for_each(|dev| {
                info!(
                    "pci: {:02x}:{:02x}.{} {:#x} {:#x} ({} {}) irq: {}:{:?}",
                    dev.loc.bus,
                    dev.loc.device,
                    dev.loc.function,
                    dev.id.vendor_id,
                    dev.id.device_id,
                    dev.id.class,
                    dev.id.subclass,
                    dev.pic_interrupt_line,
                    dev.interrupt_pin
                );
                dev.bars.iter().enumerate().for_each(|(index, bar)| {
                    if let Some(BAR::Memory(pa, len, _, t)) = bar {
                        info!("bar#{} (MMIO) {:#x} [{:#x}] [{:?}]", index, pa, len, t);
                    } else if let Some(BAR::IO(pa, len)) = bar {
                        info!("bar#{} (IO) {:#x} [{:#x}]", index, pa, len);
                    }
                });
                // match (dev.id.vendor_id,dev.id.device_id,dev.id.class,dev.id.subclass) {
                //     // ac97 audio
                //     (0x8086,0x2415,0x04,0x01) => {
                //         if let Some(BAR::IO(pa,len)) = dev.bars[0]{
                //             info!("pci: ac97 audio at {:#x} len {:#x}",pa,len);
                //             init_ac97(pa as usize);
                //         }
                //     }
                //     _ => {
                //         info!("pci: unknown device");
                //     }
                // }
            })
        }
    }
}
