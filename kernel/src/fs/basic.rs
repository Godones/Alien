use crate::fs::vfs::VfsProvider;
use crate::option_unwrap;
use crate::task::current_task;
use alloc::vec;
use pconst::LinuxErrno;
use rvfs::file::{vfs_llseek, vfs_read_file, vfs_write_file, SeekFrom};
use syscall_table::syscall_func;

#[syscall_func(285)]
pub fn copy_file_range(
    fd_in: isize,
    off_in_: usize,
    fd_out: isize,
    off_out_: usize,
    len: usize,
    _flag: usize,
) -> isize {
    warn!(
        "copy_file_range: {:?} {:?} {:?} {:?} {:?}",
        fd_in, off_in_, fd_out, off_out_, len
    );
    let task = current_task().unwrap();
    let fd_in = task.get_file(fd_in as usize);

    option_unwrap!(fd_in, LinuxErrno::EBADF.into());
    let file_in = fd_in.get_file();
    let fd_out = task.get_file(fd_out as usize);
    option_unwrap!(fd_out, LinuxErrno::EBADF.into());
    let file_out = fd_out.get_file();
    let (off_in, update_in) = if off_in_ == 0 {
        (file_in.access_inner().f_pos, true)
    } else {
        let off_in = task
            .access_inner()
            .transfer_raw_ptr(off_in_ as *const usize);
        (*off_in, false)
    };
    let (off_out, update_out) = if off_out_ == 0 {
        (file_out.access_inner().f_pos, true)
    } else {
        let off_out = task
            .access_inner()
            .transfer_raw_ptr(off_out_ as *const usize);
        (*off_out, false)
    };
    let file_in_size = file_in
        .f_dentry
        .access_inner()
        .d_inode
        .access_inner()
        .file_size;
    warn!(
        "off_in:{}, update_in:{}, off_out:{}, update_out:{}, file_in_size:{}",
        off_in, update_in, off_out, update_out, file_in_size
    );
    if off_in >= file_in_size {
        return 0;
    }

    let old_off_in = file_in.access_inner().f_pos;
    let old_off_out = file_out.access_inner().f_pos;

    // update file offset
    vfs_llseek(file_in.clone(), SeekFrom::Current(off_in as i64)).unwrap();
    vfs_llseek(file_out.clone(), SeekFrom::Current(off_out as i64)).unwrap();
    let mut buf = vec![0u8; len];
    let read_in = vfs_read_file::<VfsProvider>(file_in.clone(), &mut buf, off_in as u64);
    if read_in.is_err() {
        error!("copy_file_range: {:?}", read_in);
        return LinuxErrno::EIO.into();
    }
    let read_in = read_in.unwrap();
    warn!("read_in:{}", read_in);
    if read_in == 0 {
        file_in.access_inner().f_pos = old_off_in;
        file_out.access_inner().f_pos = old_off_out;
        return 0;
    }
    let write_out =
        vfs_write_file::<VfsProvider>(file_out.clone(), &buf[..read_in], off_out as u64);
    if write_out.is_err() {
        error!("copy_file_range: {:?}", write_out);
        return LinuxErrno::EIO.into();
    }
    let write_out = write_out.unwrap();
    warn!(
        "write_out:{}, now_off:{}",
        write_out,
        file_out.access_inner().f_pos
    );
    if !update_in {
        file_in.access_inner().f_pos = old_off_in;
        let off_in_ref = task
            .access_inner()
            .transfer_raw_ptr_mut(off_in_ as *mut usize);
        *off_in_ref = off_in + read_in;
    }
    if !update_out {
        file_out.access_inner().f_pos = old_off_out;
        let off_out_ref = task
            .access_inner()
            .transfer_raw_ptr_mut(off_out_ as *mut usize);
        error!("file_out now off:{}", file_out.access_inner().f_pos);
        *off_out_ref = off_out + write_out;
    }
    if read_in != write_out {
        panic!("copy_file_range: read_in != write_out");
    }
    read_in as isize
}
