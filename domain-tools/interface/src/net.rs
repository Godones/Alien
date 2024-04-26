use constants::AlienResult;

use crate::Basic;

pub trait NetDomain: Basic {
    fn init(&self) -> AlienResult<()>;
}
