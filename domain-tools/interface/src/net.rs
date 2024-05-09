use constants::AlienResult;
use gproxy::proxy;

use crate::Basic;
#[proxy(NetDomainProxy)]
pub trait NetDomain: Basic {
    fn init(&self, nic_domain_name: &str) -> AlienResult<()>;
}
