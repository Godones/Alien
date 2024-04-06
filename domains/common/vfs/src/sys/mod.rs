use alloc::sync::Arc;

use vfscore::dentry::VfsDentry;

pub fn init_sysfs(_root_dt: &Arc<dyn VfsDentry>) {
    basic::println!("sysfs init success");
}
