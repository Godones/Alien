use alloc::sync::Arc;
use core::fmt::Debug;

use bpf_basic::{
    linux_bpf::bpf_attr,
    prog::{BpfProgMeta, BpfProgVerifierInfo},
    EBPFPreProcessor,
};
use constants::{io::SeekFrom, AlienError, AlienResult};
use vfs::kfile::File;
use vfscore::{dentry::VfsDentry, inode::VfsInode, utils::VfsFileStat};

use crate::{
    ebpf::{map::BpfMap, transform::EbpfKernelAuxiliary},
    task::current_task,
};

pub struct BpfProg {
    meta: BpfProgMeta,
    preprocessor: EBPFPreProcessor,
}

impl Debug for BpfProg {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("BpfProg").field("meta", &self.meta).finish()
    }
}

impl BpfProg {
    pub fn new(meta: BpfProgMeta, preprocessor: EBPFPreProcessor) -> Self {
        Self { meta, preprocessor }
    }

    pub fn insns(&self) -> &[u8] {
        &self.preprocessor.get_new_insn()
    }
}

impl Drop for BpfProg {
    fn drop(&mut self) {
        unsafe {
            for ptr in self.preprocessor.get_raw_file_ptr() {
                let file = Arc::from_raw(*ptr as *const u8 as *const BpfMap);
                drop(file)
            }
        }
    }
}

impl File for BpfProg {
    fn read(&self, _buf: &mut [u8]) -> AlienResult<usize> {
        panic!("BpfProg::read() should not be called");
    }

    fn write(&self, _buf: &[u8]) -> AlienResult<usize> {
        panic!("BpfProg::write() should not be called");
    }

    fn seek(&self, _pos: SeekFrom) -> AlienResult<u64> {
        panic!("BpfProg::seek() should not be called");
    }

    fn get_attr(&self) -> AlienResult<VfsFileStat> {
        let mut stat = VfsFileStat::default();
        stat.st_size = self.preprocessor.get_new_insn().len() as u64;
        stat.st_blksize = 8;
        stat.st_blocks = (stat.st_size + (stat.st_blksize as u64) - 1) / stat.st_blksize as u64;
        Ok(stat)
    }

    fn dentry(&self) -> Arc<dyn VfsDentry> {
        panic!("BpfProg::dentry() should not be called");
    }

    fn inode(&self) -> Arc<dyn VfsInode> {
        panic!("BpfProg::inode() should not be called");
    }

    fn is_readable(&self) -> bool {
        false
    }

    fn is_writable(&self) -> bool {
        false
    }

    fn is_append(&self) -> bool {
        false
    }
}

/// Load a BPF program into the kernel.
///
/// See https://ebpf-docs.dylanreimerink.nl/linux/syscall/BPF_PROG_LOAD/
pub fn bpf_prog_load(attr: &bpf_attr) -> AlienResult<isize> {
    let mut args = BpfProgMeta::try_from_bpf_attr::<EbpfKernelAuxiliary>(attr)
        .map_err(|_| AlienError::EINVAL)?;
    println_color!(32, "bpf_prog_load: {:#?}", args);
    let _log_info = BpfProgVerifierInfo::from(attr);
    let prog_insn = args.take_insns().unwrap();
    let preprocessor =
        EBPFPreProcessor::preprocess::<EbpfKernelAuxiliary>(prog_insn).expect("preprocess failed");
    let prog = Arc::new(BpfProg::new(args, preprocessor));
    let task = current_task().unwrap();
    let fd = task
        .add_file(prog.clone())
        .map_err(|_| AlienError::EMFILE)?;
    println_color!(32, "bpf_prog_load: fd: {}", fd);
    Ok(fd as _)
}
