use alloc::sync::Arc;
use alloc::vec;
use constants::AlienResult;
use core2::io::Read;
use cpio_reader::Mode;
use vfscore::dentry::VfsDentry;
use vfscore::path::VfsPath;
use vfscore::utils::{VfsInodeMode, VfsNodeType};

pub fn populate_initrd(root: Arc<dyn VfsDentry>) -> AlienResult<()> {
    root.inode()?
        .create("bin", VfsNodeType::Dir, "rwxr-xr-x".into(), None)?;
    root.inode()?
        .create("sbin", VfsNodeType::Dir, "rwxr-xr-x".into(), None)?;
    parse_initrd_data(root)?;
    println!("Initrd populate success");
    Ok(())
}
fn parse_initrd_data(root: Arc<dyn VfsDentry>) -> AlienResult<()> {
    let mut guard = mem::data::INITRD_DATA.lock();
    if guard.is_some() {
        let path = VfsPath::new(root.clone(), root.clone());
        let data = guard.as_ref().unwrap();
        let st = data.data_ptr;
        let size = data.size;
        let data = unsafe { core::slice::from_raw_parts(st as *const u8, size) };
        let mut decoder = libflate::gzip::Decoder::new(data).unwrap();
        let mut buf = vec![];
        let _r = decoder.read_to_end(&mut buf).unwrap();
        for entry in cpio_reader::iter_files(&buf) {
            let mode = entry.mode();
            let name = entry.name();
            if name.starts_with("bin/") | name.starts_with("sbin/") {
                let inode_mode = VfsInodeMode::from_bits_truncate(mode.bits());
                if mode.contains(Mode::SYMBOLIK_LINK) {
                    // create symlink
                    let data = entry.file();
                    let target = core::str::from_utf8(data).unwrap();
                    path.join(name)?.symlink(target)?;
                } else if mode.contains(Mode::REGULAR_FILE) {
                    // create file
                    let f = path.join(name)?.open(Some(inode_mode))?;
                    f.inode()?.write_at(0, entry.file())?;
                }
            }
        }
        // release the page frame
        guard.take();
    }
    Ok(())
}
