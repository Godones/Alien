use alloc::sync::Arc;

use spin::Once;
use vfscore::dentry::VfsDentry;

pub static PIPE_FS_ROOT: Once<Arc<dyn VfsDentry>> = Once::new();

pub fn init_pipefs(root_dt: &Arc<dyn VfsDentry>) {
    PIPE_FS_ROOT.call_once(|| root_dt.clone());
    basic::println!("pipefs init success");
}
