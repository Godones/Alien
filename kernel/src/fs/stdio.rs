use alloc::sync::Arc;

use lazy_static::lazy_static;
use rvfs::dentry::DirEntry;
use rvfs::file::{File, FileMode, FileOps, OpenFlags};
use rvfs::mount::VfsMount;
use rvfs::StrResult;

use crate::print::console::get_char;

type Stdin = File;
type Stdout = File;

lazy_static! {
    pub static ref STDIN: Arc<Stdin> = {
        let mut file_ops = FileOps::empty();
        file_ops.read = stdin_read;
        let file = File::new(
            Arc::new(DirEntry::empty()),
            Arc::new(VfsMount::empty()),
            OpenFlags::O_RDONLY,
            FileMode::FMODE_READ,
            file_ops,
        );
        Arc::new(file)
    };
}

fn stdin_read(_file: Arc<File>, buf: &mut [u8], _offset: u64) -> StrResult<usize> {
    return match get_char() {
        Some(ch) => {
            if ch == 13 {
                buf[0] = 10;
                return Ok(1);
            }
            buf[0] = ch;
            Ok(1)
        }
        None => Ok(0),
    };
}
lazy_static! {
    pub static ref STDOUT: Arc<Stdout> = {
        let mut file_ops = FileOps::empty();
        file_ops.write = stdout_write;
        let file = File::new(
            Arc::new(DirEntry::empty()),
            Arc::new(VfsMount::empty()),
            OpenFlags::O_WRONLY,
            FileMode::FMODE_WRITE,
            file_ops,
        );
        Arc::new(file)
    };
}

fn stdout_write(_file: Arc<File>, buf: &[u8], _offset: u64) -> StrResult<usize> {
    uprint!("{}", core::str::from_utf8(buf).unwrap());
    Ok(buf.len())
}
