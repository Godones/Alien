mod device;

use device::PciBus;
use ksync::Mutex;

use crate::bus::CommonDeviceInfo;

static PCI_BUS: Mutex<PciBus> = Mutex::new(PciBus::new());

pub fn pci_init(pci_info: CommonDeviceInfo) {}
