mod blk;
pub mod continuation;
mod shadow_blk;

use core::ops::Range;

pub use blk::*;
use constants::{io::RtcTime, AlienError, AlienResult};
use interface::*;
use rref::{RRef, RRefVec};
pub use shadow_blk::*;
use task_meta::TaskMeta;
use vfscore::{fstype::FileSystemFlags, inode::InodeAttr, superblock::SuperType, utils::*};
gen_for_BufInputDomain!();
gen_for_BufUartDomain!();
gen_for_CacheBlkDeviceDomain!();
gen_for_EmptyDeviceDomain!();
gen_for_FsDomain!();
gen_for_GpuDomain!();
gen_for_InputDomain!();
gen_for_NetDomain!();
gen_for_RtcDomain!();
// gen_for_ShadowBlockDomain!();
gen_for_SysCallDomain!();
gen_for_TaskDomain!();
gen_for_UartDomain!();
gen_for_VfsDomain!();
gen_for_PLICDomain!();
gen_for_SchedulerDomain!();
impl Basic for DevFsDomainProxy {
    fn is_active(&self) -> bool {
        self.domain.is_active()
    }
}
gen_for_DevFsDomain!();
impl_for_FsDomain!(DevFsDomainProxy);
