use alloc::boxed::Box;
use alloc::sync::Arc;

use lazy_static::lazy_static;
use rvfs::dentry::DirEntry;
use rvfs::file::{File, FileExtOps, FileMode, FileOps, OpenFlags};
use rvfs::inode::SpecialData;
use rvfs::mount::VfsMount;
use rvfs::superblock::{DataOps, Device};
use rvfs::StrResult;

use syscall_define::io::{LocalModes, TeletypeCommand, Termios, WinSize};

use crate::print::console::{check_have_char, get_char};
use crate::task::current_task;

type Stdin = File;
type Stdout = File;

#[derive(Debug, Default)]
pub struct IoData {
    foreground_pgid: u32,
    winsize: WinSize,
    termios: Termios,
}

impl DataOps for IoData {
    fn device(&self, _name: &str) -> Option<Arc<dyn Device>> {
        None
    }
    fn data(&self) -> *const u8 {
        self as *const _ as *const u8
    }
}

impl IoData {
    pub fn from_ptr(ptr: *const u8) -> &'static mut Self {
        unsafe { &mut *(ptr as *mut Self) }
    }
}

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
        file.access_inner().f_ops_ext = {
            let mut ext_ops = FileExtOps::empty();
            ext_ops.is_ready_read = stdin_ready_to_read;
            ext_ops.is_ready_write = |_| false;
            ext_ops.ioctl = stdin_ioctl;
            ext_ops
        };
        file.f_dentry.access_inner().d_inode.access_inner().data =
            Some(Box::new(IoData::default()));
        file.f_dentry
            .access_inner()
            .d_inode
            .access_inner()
            .special_data = Some(SpecialData::CharData(0 as *const u8));
        Arc::new(file)
    };
}

fn stdin_ready_to_read(_file: Arc<File>) -> bool {
    check_have_char()
}

fn stdin_read(file: Arc<File>, buf: &mut [u8], _offset: u64) -> StrResult<usize> {
    let binding = file.f_dentry.access_inner();
    let inode_inner = binding.d_inode.access_inner();
    let data = inode_inner.data.as_ref().unwrap();
    let data = IoData::from_ptr(data.data());

    // read util \r and transform to \n
    let mut read_count = 0;
    loop {
        let ch = get_char();
        assert!(ch.is_some());
        let ch = ch.unwrap();
        buf[read_count] = ch;
        read_count += 1;
        if ch == b'\r' {
            buf[read_count - 1] = b'\n';
            if LocalModes::from_bits_truncate(data.termios.lflag).contains(LocalModes::ECHO) {
                uprint!("\n");
            }
            break;
        }
        if LocalModes::from_bits_truncate(data.termios.lflag).contains(LocalModes::ECHO) {
            uprint!("{}", ch as char);
        }
        if read_count >= buf.len() {
            break;
        }
    }
    Ok(read_count)
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
        file.access_inner().f_ops_ext = {
            let mut ext_ops = FileExtOps::empty();
            ext_ops.is_ready_read = |_| false;
            ext_ops.is_ready_write = |_| true;
            ext_ops.ioctl = stdin_ioctl;
            ext_ops
        };
        file.f_dentry.access_inner().d_inode.access_inner().data =
            Some(Box::new(IoData::default()));
        file.f_dentry
            .access_inner()
            .d_inode
            .access_inner()
            .special_data = Some(SpecialData::CharData(0 as *const u8));
        Arc::new(file)
    };
}

fn stdout_write(_file: Arc<File>, buf: &[u8], _offset: u64) -> StrResult<usize> {
    uprint!("{}", core::str::from_utf8(buf).unwrap());
    Ok(buf.len())
}

fn stdin_ioctl(file: Arc<File>, cmd: u32, args: usize) -> isize {
    let task = current_task().unwrap();
    let mut task_inner = task.access_inner();
    let binding = file.f_dentry.access_inner();
    let inode_inner = binding.d_inode.access_inner();
    let data = inode_inner.data.as_ref().unwrap();
    let data = IoData::from_ptr(data.data());
    let cmd = TeletypeCommand::try_from(cmd).unwrap();
    match cmd {
        TeletypeCommand::TCGETS | TeletypeCommand::TCGETA => {
            task_inner.copy_to_user(&data.termios, args as *mut Termios);
            0
        }
        TeletypeCommand::TCSETS | TeletypeCommand::TCSETSW | TeletypeCommand::TCSETSF => {
            task_inner.copy_from_user(args as *const Termios, &mut data.termios);
            0
        }
        TeletypeCommand::TIOCGPGRP => {
            let word = task_inner.transfer_raw_ptr_mut(args as *mut u32);
            *word = data.foreground_pgid;
            0
        }
        TeletypeCommand::TIOCSPGRP => {
            let word = task_inner.transfer_raw_ptr(args as *const u32);
            data.foreground_pgid = *word;
            0
        }
        TeletypeCommand::TIOCGWINSZ => {
            task_inner.copy_to_user(&data.winsize, args as *mut WinSize);
            0
        }
        TeletypeCommand::TIOCSWINSZ => {
            task_inner.copy_from_user(args as *const WinSize, &mut data.winsize);
            0
        }
        _ => {
            unimplemented!("ioctl cmd: {:?}", cmd)
        }
    }
}
