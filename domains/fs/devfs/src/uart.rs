use alloc::sync::Arc;

use basic::{
    constants::{
        io::{LocalModes, TeletypeCommand, Termios, WinSize},
        DeviceId,
    },
    sync::Mutex,
};
use interface::{BufUartDomain, TaskDomain};
use pod::Pod;
use vfscore::{
    error::VfsError,
    file::VfsFile,
    inode::{InodeAttr, VfsInode},
    superblock::VfsSuperBlock,
    utils::{VfsFileStat, VfsNodeType, VfsPollEvents},
    VfsResult,
};
#[derive(Debug, Default)]
pub struct IoData {
    foreground_pgid: u32,
    winsize: WinSize,
    termios: Termios,
}

pub struct UARTDevice {
    device_id: DeviceId,
    device: Arc<dyn BufUartDomain>,
    io: Mutex<IoData>,
    task_domain: Arc<dyn TaskDomain>,
}

impl UARTDevice {
    pub fn new(
        device_id: DeviceId,
        device: Arc<dyn BufUartDomain>,
        task: Arc<dyn TaskDomain>,
    ) -> Self {
        Self {
            device_id,
            device,
            io: Mutex::new(IoData::default()),
            task_domain: task,
        }
    }
}

impl VfsFile for UARTDevice {
    fn read_at(&self, _offset: u64, buf: &mut [u8]) -> VfsResult<usize> {
        // read util \r and transform to \n
        let mut read_count = 0;
        loop {
            let ch = self.device.getc().unwrap();
            assert!(ch.is_some());
            let ch = ch.unwrap();
            buf[read_count] = ch;
            read_count += 1;
            if ch == b'\r' {
                buf[read_count - 1] = b'\n';
                if LocalModes::from_bits_truncate(self.io.lock().termios.lflag)
                    .contains(LocalModes::ECHO)
                {
                    self.device.putc(b'\n').unwrap();
                }
                break;
            }
            if LocalModes::from_bits_truncate(self.io.lock().termios.lflag)
                .contains(LocalModes::ECHO)
            {
                self.device.putc(ch).unwrap();
            }
            if read_count >= buf.len() {
                break;
            }
        }
        Ok(read_count)
    }
    fn write_at(&self, _offset: u64, buf: &[u8]) -> VfsResult<usize> {
        buf.iter().for_each(|c| self.device.putc(*c).unwrap());
        Ok(buf.len())
    }
    fn poll(&self, event: VfsPollEvents) -> VfsResult<VfsPollEvents> {
        let mut res = VfsPollEvents::empty();
        if event.contains(VfsPollEvents::IN) {
            if self.device.have_data_to_get().unwrap() {
                res |= VfsPollEvents::IN;
            }
        }
        if event.contains(VfsPollEvents::OUT) {
            if self.device.have_space_to_put().unwrap() {
                res |= VfsPollEvents::OUT
            }
        }
        Ok(res)
    }
    fn ioctl(&self, cmd: u32, arg: usize) -> VfsResult<usize> {
        let mut io = self.io.lock();
        let cmd = TeletypeCommand::try_from(cmd).unwrap();
        return match cmd {
            TeletypeCommand::TCGETS | TeletypeCommand::TCGETA => {
                self.task_domain
                    .copy_to_user(arg, &io.termios.as_bytes())
                    .unwrap();
                Ok(0)
            }
            TeletypeCommand::TCSETS | TeletypeCommand::TCSETSW | TeletypeCommand::TCSETSF => {
                let buf = io.termios.as_bytes_mut();
                self.task_domain.copy_from_user(arg, buf).unwrap();
                Ok(0)
            }
            TeletypeCommand::TIOCGPGRP => {
                self.task_domain
                    .write_val_to_user(arg, &io.foreground_pgid)
                    .unwrap();
                Ok(0)
            }
            TeletypeCommand::TIOCSPGRP => {
                let word = self.task_domain.read_val_from_user(arg).unwrap();
                io.foreground_pgid = word;
                Ok(0)
            }
            TeletypeCommand::TIOCGWINSZ => {
                self.task_domain
                    .copy_to_user(arg, io.winsize.as_bytes())
                    .unwrap();
                Ok(0)
            }
            TeletypeCommand::TIOCSWINSZ => {
                self.task_domain
                    .copy_from_user(arg, io.winsize.as_bytes_mut())
                    .unwrap();
                Ok(0)
            }
            _ => {
                unimplemented!("ioctl cmd: {:?}", cmd)
            }
        };
    }
    fn flush(&self) -> VfsResult<()> {
        Ok(())
    }
    fn fsync(&self) -> VfsResult<()> {
        Ok(())
    }
}

impl VfsInode for UARTDevice {
    fn get_super_block(&self) -> VfsResult<Arc<dyn VfsSuperBlock>> {
        Err(VfsError::NoSys)
    }

    fn set_attr(&self, _attr: InodeAttr) -> VfsResult<()> {
        Ok(())
    }

    fn get_attr(&self) -> VfsResult<VfsFileStat> {
        Ok(VfsFileStat {
            st_rdev: self.device_id.id(),
            ..Default::default()
        })
    }

    fn inode_type(&self) -> VfsNodeType {
        VfsNodeType::CharDevice
    }
}
