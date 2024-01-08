use fdt::node::FdtNode;
use fdt::Fdt;

pub fn get_device_info(fdt: &Fdt, device_name: &str) -> Option<(usize, usize)> {
    let res = fdt
        .all_nodes()
        .find(|node| node.name.starts_with(device_name));
    let res = res.and_then(|node| {
        if let Some(reg) = node.reg().and_then(|mut reg| reg.next()) {
            let addr = reg.starting_address as usize;
            if let Some(mut interrupts) = node.interrupts() {
                let irq = interrupts.next().unwrap();
                return Some((addr, irq));
            } else {
                None
            }
        } else {
            None
        }
    });
    res
}

#[allow(unused)]
pub fn get_device_info_from_node(node: &FdtNode) -> Option<(usize, usize)> {
    if let Some(reg) = node.reg().and_then(|mut reg| reg.next()) {
        let addr = reg.starting_address as usize;
        if let Some(mut interrupts) = node.interrupts() {
            let irq = interrupts.next().unwrap();
            Some((addr, irq))
        } else {
            None
        }
    } else {
        None
    }
}
