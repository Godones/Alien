use alloc::sync::Arc;

use fatfs::{IoBase, Read, Seek, SeekFrom, Write};
use vfscore::{inode::VfsInode, RRefVec};

#[derive(Clone)]
pub struct FatDevice {
    pub pos: i64,
    pub device_file: Arc<dyn VfsInode>,
}

impl FatDevice {
    pub fn new(device: Arc<dyn VfsInode>) -> Self {
        Self {
            pos: 0,
            device_file: device,
        }
    }
}

impl IoBase for FatDevice {
    type Error = ();
}
impl Write for FatDevice {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        let rvec = RRefVec::from_other_rvec_slice(buf);
        let len = self
            .device_file
            .write_at(self.pos as u64, &rvec)
            .map_err(|_| ())?;
        self.pos += len as i64;
        Ok(len)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.device_file.flush().map_err(|_| ())
    }
}

impl Read for FatDevice {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        let rvec = RRefVec::from_other_rvec_slice(buf);
        let (_buf, len) = self
            .device_file
            .read_at(self.pos as u64, rvec)
            .map_err(|_| ())?;
        self.pos += len as i64;
        Ok(len)
    }
}

impl Seek for FatDevice {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, Self::Error> {
        let pos = match pos {
            SeekFrom::Start(pos) => pos as i64,
            SeekFrom::End(pos) => {
                let len = self.device_file.get_attr().unwrap().st_size;
                len as i64 + pos
            }
            SeekFrom::Current(pos) => self.pos + pos,
        };
        if pos < 0 {
            return Err(());
        }
        self.pos = pos;
        Ok(pos as u64)
    }
}
