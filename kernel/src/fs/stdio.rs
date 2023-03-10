use crate::print::console::get_char;
use alloc::sync::Arc;
use lazy_static::lazy_static;
use rvfs::dentry::DirEntry;
use rvfs::file::{File, FileFlags, FileMode, FileOps};
use rvfs::mount::VfsMount;
use rvfs::StrResult;

type Stdin = File;
type Stdout = File;

lazy_static! {
    pub static ref STDIN: Arc<Stdin> = {
        let mut file_ops = FileOps::empty();
        file_ops.read = stdin_read;
        let file = File::new(
            Arc::new(DirEntry::empty()),
            Arc::new(VfsMount::empty()),
            FileFlags::O_RDONLY,
            FileMode::FMODE_READ,
            file_ops,
        );
        Arc::new(file)
    };
}

fn stdin_read(_file: Arc<File>, buf: &mut [u8], _offset: u64) -> StrResult<usize> {
    assert_eq!(buf.len(), 1);
    loop {
        match get_char() {
            Some(ch) => {
                buf[0] = ch as u8;
                return Ok(1);
            }
            None => {}
        }
    }
}
lazy_static! {
    pub static ref STDOUT: Arc<Stdout> = {
        let mut file_ops = FileOps::empty();
        file_ops.write = stdout_write;
        let file = File::new(
            Arc::new(DirEntry::empty()),
            Arc::new(VfsMount::empty()),
            FileFlags::O_WRONLY,
            FileMode::FMODE_WRITE,
            file_ops,
        );
        Arc::new(file)
    };
}

fn stdout_write(_file: Arc<File>, buf: &[u8], _offset: u64) -> StrResult<usize> {
    print!("{}", core::str::from_utf8(buf).unwrap());
    Ok(buf.len())
}
