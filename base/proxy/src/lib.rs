#![no_std]
#![feature(linkage)]
#![feature(naked_functions)]
extern crate alloc;

mod trampoline;

use alloc::sync::Arc;
use constants::io::FileStat;
use constants::{AlienError, AlienResult};
use core::arch::asm;
use core::fmt::Debug;
use domain_loader::DomainLoader;
use interface::*;
use ksync::{Mutex, RwLock};
use rref::{RRef, RRefVec};

#[derive(Debug)]
pub struct BlkDomainProxy {
    domain_id: u64,
    domain: RwLock<Arc<dyn BlkDeviceDomain>>,
    domain_loader: DomainLoader,
    device_info: Mutex<Option<DeviceInfo>>,
}

impl BlkDomainProxy {
    pub fn new(
        domain_id: u64,
        domain: Arc<dyn BlkDeviceDomain>,
        domain_loader: DomainLoader,
    ) -> Self {
        Self {
            domain_id,
            domain: RwLock::new(domain),
            domain_loader,
            device_info: Mutex::new(None),
        }
    }
}

impl Basic for BlkDomainProxy {
    fn is_active(&self) -> bool {
        self.domain.read().is_active()
    }
}

impl DeviceBase for BlkDomainProxy {
    fn handle_irq(&self) -> AlienResult<()> {
        if !self.is_active() {
            return Err(AlienError::DOMAINCRASH);
        }
        self.domain.read().handle_irq()
    }
}

impl BlkDeviceDomain for BlkDomainProxy {
    fn init(&self, device_info: &DeviceInfo) -> AlienResult<()> {
        self.device_info.lock().replace(device_info.clone());
        self.domain.read().init(device_info)
    }

    fn read_block(&self, block: u32, data: RRef<[u8; 512]>) -> AlienResult<RRef<[u8; 512]>> {
        if !self.is_active() {
            return Err(AlienError::DOMAINCRASH);
        }
        // self.domain.read(block, data)
        let res = {
            let guard = self.domain.read();
            unsafe { blk_domain_proxy_read_trampoline(&guard, block, data) }
        };
        res
    }
    fn write_block(&self, block: u32, data: &RRef<[u8; 512]>) -> AlienResult<usize> {
        if !self.is_active() {
            return Err(AlienError::DOMAINCRASH);
        }
        self.domain.read().write_block(block, data)
    }
    fn get_capacity(&self) -> AlienResult<u64> {
        if !self.is_active() {
            return Err(AlienError::DOMAINCRASH);
        }
        self.domain.read().get_capacity()
    }
    fn flush(&self) -> AlienResult<()> {
        if !self.is_active() {
            return Err(AlienError::DOMAINCRASH);
        }
        self.domain.read().flush()
    }

    // todo!()
    fn restart(&self) -> bool {
        let mut domain = self.domain.write();
        self.domain_loader.reload().unwrap();
        // let mut loader = DomainLoader::new(self.domain_loader.data());
        // loader.load().unwrap();
        // let new_domain = loader.call(self.domain_id);

        let mut new_domain = self
            .domain_loader
            .call::<dyn BlkDeviceDomain>(self.domain_id);
        let device_info = self.device_info.lock();
        new_domain.init(device_info.as_ref().unwrap()).unwrap();
        core::mem::swap(&mut *domain, &mut new_domain);
        // The new_domain now is the old domain, but it has been recycled so we
        // can't drop it again
        core::mem::forget(new_domain);
        true
    }
}
#[naked]
#[no_mangle]
#[allow(undefined_naked_function_abi)]
unsafe fn blk_domain_proxy_read_trampoline(
    blk_domain: &Arc<dyn BlkDeviceDomain>,
    block: u32,
    data: RRef<[u8; 512]>,
) -> AlienResult<RRef<[u8; 512]>> {
    asm!(
        "addi sp, sp, -33*8",
        "sd x0, 0*8(sp)",
        "sd x1, 1*8(sp)",
        "sd x2, 2*8(sp)",
        "sd x3, 3*8(sp)",
        "sd x4, 4*8(sp)",
        "sd x5, 5*8(sp)",
        "sd x6, 6*8(sp)",
        "sd x7, 7*8(sp)",
        "sd x8, 8*8(sp)",
        "sd x9, 9*8(sp)",
        "sd x10, 10*8(sp)",
        "sd x11, 11*8(sp)",
        "sd x12, 12*8(sp)",
        "sd x13, 13*8(sp)",
        "sd x14, 14*8(sp)",
        "sd x15, 15*8(sp)",
        "sd x16, 16*8(sp)",
        "sd x17, 17*8(sp)",
        "sd x18, 18*8(sp)",
        "sd x19, 19*8(sp)",
        "sd x20, 20*8(sp)",
        "sd x21, 21*8(sp)",
        "sd x22, 22*8(sp)",
        "sd x23, 23*8(sp)",
        "sd x24, 24*8(sp)",
        "sd x25, 25*8(sp)",
        "sd x26, 26*8(sp)",
        "sd x27, 27*8(sp)",
        "sd x28, 28*8(sp)",
        "sd x29, 29*8(sp)",
        "sd x30, 30*8(sp)",
        "sd x31, 31*8(sp)",
        "call blk_domain_proxy_read_ptr",
        "sd a0, 32*8(sp)",
        "mv a0, sp",
        "call register_cont",
        //  recover caller saved registers
        "ld ra, 1*8(sp)",
        "ld x5, 5*8(sp)",
        "ld x6, 6*8(sp)",
        "ld x7, 7*8(sp)",
        "ld x10, 10*8(sp)",
        "ld x11, 11*8(sp)",
        "ld x12, 12*8(sp)",
        "ld x13, 13*8(sp)",
        "ld x14, 14*8(sp)",
        "ld x15, 15*8(sp)",
        "ld x16, 16*8(sp)",
        "ld x17, 17*8(sp)",
        "ld x28, 28*8(sp)",
        "ld x29, 29*8(sp)",
        "ld x30, 30*8(sp)",
        "ld x31, 31*8(sp)",
        "addi sp, sp, 33*8",
        "la gp, blk_domain_proxy_read",
        "jr gp",
        options(noreturn)
    )
}

#[no_mangle]
fn blk_domain_proxy_read(
    blk_domain: &Arc<dyn BlkDeviceDomain>,
    block: u32,
    data: RRef<[u8; 512]>,
) -> AlienResult<RRef<[u8; 512]>> {
    // info!("BlkDomainProxy_read");
    blk_domain.read_block(block, data)
}
#[no_mangle]
fn blk_domain_proxy_read_err() -> AlienResult<RRef<[u8; 512]>> {
    platform::println!("BlkDomainProxy_read should return error");
    Err(AlienError::DOMAINCRASH)
}

#[no_mangle]
fn blk_domain_proxy_read_ptr() -> usize {
    blk_domain_proxy_read_err as usize
}

#[derive(Debug)]
pub struct ShadowBlockDomainProxy {
    domain_id: u64,
    domain: Arc<dyn ShadowBlockDomain>,
}

impl ShadowBlockDomainProxy {
    pub fn new(domain_id: u64, domain: Arc<dyn ShadowBlockDomain>) -> Self {
        Self { domain_id, domain }
    }
}

impl DeviceBase for ShadowBlockDomainProxy {
    fn handle_irq(&self) -> AlienResult<()> {
        if !self.is_active() {
            return Err(AlienError::DOMAINCRASH);
        }
        self.domain.handle_irq()
    }
}

impl Basic for ShadowBlockDomainProxy {
    fn is_active(&self) -> bool {
        self.domain.is_active()
    }
}

impl ShadowBlockDomain for ShadowBlockDomainProxy {
    fn init(&self, blk_domain: &str) -> AlienResult<()> {
        if !self.is_active() {
            return Err(AlienError::DOMAINCRASH);
        }
        self.domain.init(blk_domain)
    }

    fn read_block(&self, block: u32, data: RRef<[u8; 512]>) -> AlienResult<RRef<[u8; 512]>> {
        if !self.is_active() {
            return Err(AlienError::DOMAINCRASH);
        }
        self.domain.read_block(block, data)
    }

    fn write_block(&self, block: u32, data: &RRef<[u8; 512]>) -> AlienResult<usize> {
        if !self.is_active() {
            return Err(AlienError::DOMAINCRASH);
        }
        self.domain.write_block(block, data)
    }

    fn get_capacity(&self) -> AlienResult<u64> {
        if !self.is_active() {
            return Err(AlienError::DOMAINCRASH);
        }
        self.domain.get_capacity()
    }

    fn flush(&self) -> AlienResult<()> {
        if !self.is_active() {
            return Err(AlienError::DOMAINCRASH);
        }
        self.domain.flush()
    }
}

#[derive(Debug)]
pub struct FsDomainProxy {
    domain_id: u64,
    domain: Arc<dyn FsDomain>,
}

impl FsDomainProxy {
    pub fn new(domain_id: u64, domain: Arc<dyn FsDomain>) -> Self {
        Self { domain_id, domain }
    }
}

impl Basic for FsDomainProxy {
    fn is_active(&self) -> bool {
        self.domain.is_active()
    }
}

impl FsDomain for FsDomainProxy {}

#[derive(Debug)]
pub struct RtcDomainProxy {
    domain: Arc<dyn RtcDomain>,
}

impl RtcDomainProxy {
    pub fn new(_domain_id: u64, domain: Arc<dyn RtcDomain>) -> Self {
        Self { domain }
    }
}

impl Basic for RtcDomainProxy {
    fn is_active(&self) -> bool {
        self.domain.is_active()
    }
}

impl DeviceBase for RtcDomainProxy {
    fn handle_irq(&self) -> AlienResult<()> {
        if self.domain.is_active() {
            self.domain.handle_irq()
        } else {
            Err(AlienError::DOMAINCRASH)
        }
    }
}

impl RtcDomain for RtcDomainProxy {
    fn init(&self, device_info: &DeviceInfo) -> AlienResult<()> {
        self.domain.init(device_info)
    }

    fn read_time(&self, time: RRef<RtcTime>) -> AlienResult<RRef<RtcTime>> {
        if self.domain.is_active() {
            self.domain.read_time(time)
        } else {
            Err(AlienError::DOMAINCRASH)
        }
    }
}

#[derive(Debug)]
pub struct VfsDomainProxy {
    domain: Arc<dyn VfsDomain>,
}

impl VfsDomainProxy {
    pub fn new(_id: u64, domain: Arc<dyn VfsDomain>) -> Self {
        Self { domain }
    }
}

impl Basic for VfsDomainProxy {
    fn is_active(&self) -> bool {
        self.domain.is_active()
    }
}

impl VfsDomain for VfsDomainProxy {
    fn init(&self) -> AlienResult<()> {
        self.domain.init()
    }

    fn vfs_open(
        &self,
        root: InodeId,
        path: &RRefVec<u8>,
        mode: u32,
        open_flags: usize,
    ) -> AlienResult<InodeId> {
        if self.domain.is_active() {
            self.domain.vfs_open(root, path, mode, open_flags)
        } else {
            Err(AlienError::DOMAINCRASH)
        }
    }

    fn vfs_getattr(&self, inode: InodeId, attr: RRef<FileStat>) -> AlienResult<RRef<FileStat>> {
        if self.domain.is_active() {
            self.domain.vfs_getattr(inode, attr)
        } else {
            Err(AlienError::DOMAINCRASH)
        }
    }

    fn vfs_read_at(
        &self,
        inode: InodeId,
        offset: u64,
        buf: RRefVec<u8>,
    ) -> AlienResult<(RRefVec<u8>, usize)> {
        if self.domain.is_active() {
            self.domain.vfs_read_at(inode, offset, buf)
        } else {
            Err(AlienError::DOMAINCRASH)
        }
    }
    fn vfs_read(&self, inode: InodeId, buf: RRefVec<u8>) -> AlienResult<(RRefVec<u8>, usize)> {
        if self.domain.is_active() {
            self.domain.vfs_read(inode, buf)
        } else {
            Err(AlienError::DOMAINCRASH)
        }
    }
    fn vfs_write_at(
        &self,
        inode: InodeId,
        offset: u64,
        buf: RRefVec<u8>,
    ) -> AlienResult<(RRefVec<u8>, usize)> {
        if self.domain.is_active() {
            self.domain.vfs_write_at(inode, offset, buf)
        } else {
            Err(AlienError::DOMAINCRASH)
        }
    }
    fn vfs_write(&self, inode: InodeId, buf: &RRefVec<u8>) -> AlienResult<usize> {
        if self.domain.is_active() {
            self.domain.vfs_write(inode, buf)
        } else {
            Err(AlienError::DOMAINCRASH)
        }
    }
}

#[derive(Debug)]
pub struct CacheBlkDomainProxy {
    domain_id: u64,
    domain: Arc<dyn CacheBlkDeviceDomain>,
}

impl CacheBlkDomainProxy {
    pub fn new(domain_id: u64, domain: Arc<dyn CacheBlkDeviceDomain>) -> Self {
        Self { domain_id, domain }
    }
}

impl Basic for CacheBlkDomainProxy {
    fn is_active(&self) -> bool {
        self.domain.is_active()
    }
}

impl DeviceBase for CacheBlkDomainProxy {
    fn handle_irq(&self) -> AlienResult<()> {
        if !self.is_active() {
            return Err(AlienError::DOMAINCRASH);
        }
        self.domain.handle_irq()
    }
}

impl CacheBlkDeviceDomain for CacheBlkDomainProxy {
    fn init(&self, blk_domain_name: &str) -> AlienResult<()> {
        self.domain.init(blk_domain_name)
    }

    fn read(&self, offset: u64, buf: RRefVec<u8>) -> AlienResult<RRefVec<u8>> {
        if !self.is_active() {
            return Err(AlienError::DOMAINCRASH);
        }
        self.domain.read(offset, buf)
    }

    fn write(&self, offset: u64, buf: &RRefVec<u8>) -> AlienResult<usize> {
        if !self.is_active() {
            return Err(AlienError::DOMAINCRASH);
        }
        self.domain.write(offset, buf)
    }

    fn get_capacity(&self) -> AlienResult<u64> {
        if !self.is_active() {
            return Err(AlienError::DOMAINCRASH);
        }
        self.domain.get_capacity()
    }

    fn flush(&self) -> AlienResult<()> {
        if !self.is_active() {
            return Err(AlienError::DOMAINCRASH);
        }
        self.domain.flush()
    }
}

#[derive(Debug)]
pub struct EIntrDomainProxy {
    id: u64,
    domain: Arc<dyn PLICDomain>,
}

impl EIntrDomainProxy {
    pub fn new(id: u64, domain: Arc<dyn PLICDomain>) -> Self {
        Self { id, domain }
    }
}

impl Basic for EIntrDomainProxy {
    fn is_active(&self) -> bool {
        self.domain.is_active()
    }
}

impl PLICDomain for EIntrDomainProxy {
    fn init(&self, device_info: &DeviceInfo) -> AlienResult<()> {
        self.domain.init(device_info)
    }

    fn handle_irq(&self) -> AlienResult<()> {
        if !self.is_active() {
            return Err(AlienError::DOMAINCRASH);
        }
        self.domain.handle_irq()
    }
    fn register_irq(&self, irq: usize, device: Arc<dyn DeviceBase>) -> AlienResult<()> {
        if !self.is_active() {
            return Err(AlienError::DOMAINCRASH);
        }
        self.domain.register_irq(irq, device)
    }

    fn irq_info(&self, buf: RRefVec<u8>) -> AlienResult<RRefVec<u8>> {
        if !self.is_active() {
            return Err(AlienError::DOMAINCRASH);
        }
        self.domain.irq_info(buf)
    }
}

#[derive(Debug)]
pub struct DevicesDomainProxy {
    id: u64,
    domain: Arc<dyn DevicesDomain>,
}

impl DevicesDomainProxy {
    pub fn new(id: u64, domain: Arc<dyn DevicesDomain>) -> Self {
        Self { id, domain }
    }
}

impl Basic for DevicesDomainProxy {
    fn is_active(&self) -> bool {
        self.domain.is_active()
    }
}

impl DevicesDomain for DevicesDomainProxy {
    fn init(&self, dtb: &'static [u8]) -> AlienResult<()> {
        self.domain.init(dtb)
    }

    fn index_device(&self, index: usize, info: RRef<DeviceInfo>) -> AlienResult<RRef<DeviceInfo>> {
        if !self.is_active() {
            return Err(AlienError::DOMAINCRASH);
        }
        self.domain.index_device(index, info)
    }
}

#[derive(Debug)]
pub struct GpuDomainProxy {
    id: u64,
    domain: Arc<dyn GpuDomain>,
}

impl GpuDomainProxy {
    pub fn new(id: u64, domain: Arc<dyn GpuDomain>) -> Self {
        Self { id, domain }
    }
}

impl GpuDomain for GpuDomainProxy {
    fn init(&self, device_info: DeviceInfo) -> AlienResult<()> {
        self.domain.init(device_info)
    }

    fn flush(&self) -> AlienResult<()> {
        if !self.is_active() {
            return Err(AlienError::DOMAINCRASH);
        }
        self.domain.flush()
    }

    fn fill(&self, offset: u32, buf: &RRefVec<u8>) -> AlienResult<usize> {
        if !self.is_active() {
            return Err(AlienError::DOMAINCRASH);
        }
        self.domain.fill(offset, buf)
    }
}

impl DeviceBase for GpuDomainProxy {
    fn handle_irq(&self) -> AlienResult<()> {
        if !self.is_active() {
            return Err(AlienError::DOMAINCRASH);
        }
        self.domain.handle_irq()
    }
}

impl Basic for GpuDomainProxy {
    fn is_active(&self) -> bool {
        self.domain.is_active()
    }
}

#[derive(Debug)]
pub struct UartDomainProxy {
    id: u64,
    domain: Arc<dyn UartDomain>,
}

impl UartDomainProxy {
    pub fn new(id: u64, domain: Arc<dyn UartDomain>) -> Self {
        Self { id, domain }
    }
}

impl UartDomain for UartDomainProxy {
    fn init(&self, device_info: &DeviceInfo) -> AlienResult<()> {
        self.domain.init(device_info)
    }

    fn putc(&self, ch: u8) -> AlienResult<()> {
        if !self.is_active() {
            return Err(AlienError::DOMAINCRASH);
        }
        self.domain.putc(ch)
    }

    fn getc(&self) -> AlienResult<Option<u8>> {
        if !self.is_active() {
            return Err(AlienError::DOMAINCRASH);
        }
        self.domain.getc()
    }

    fn have_data_to_get(&self) -> AlienResult<bool> {
        if !self.is_active() {
            return Err(AlienError::DOMAINCRASH);
        }
        self.domain.have_data_to_get()
    }
}

impl DeviceBase for UartDomainProxy {
    fn handle_irq(&self) -> AlienResult<()> {
        if !self.is_active() {
            return Err(AlienError::DOMAINCRASH);
        }
        self.domain.handle_irq()
    }
}

impl Basic for UartDomainProxy {
    fn is_active(&self) -> bool {
        self.domain.is_active()
    }
}
#[derive(Debug)]
pub struct TaskDomainProxy {
    id: u64,
    domain: Arc<dyn TaskDomain>,
}
impl TaskDomainProxy {
    pub fn new(id: u64, domain: Arc<dyn TaskDomain>) -> Self {
        Self { id, domain }
    }
}

impl Basic for TaskDomainProxy {
    fn is_active(&self) -> bool {
        self.domain.is_active()
    }
}

impl TaskDomain for TaskDomainProxy {
    fn init(&self) -> AlienResult<()> {
        self.domain.init()
    }

    fn run(&self) {
        if !self.is_active() {
            return;
        }
        self.domain.run()
    }

    fn trap_frame_virt_addr(&self) -> AlienResult<usize> {
        if !self.is_active() {
            return Err(AlienError::DOMAINCRASH);
        }
        self.domain.trap_frame_virt_addr()
    }

    fn current_task_satp(&self) -> AlienResult<usize> {
        if !self.is_active() {
            return Err(AlienError::DOMAINCRASH);
        }
        self.domain.current_task_satp()
    }

    fn trap_frame_phy_addr(&self) -> AlienResult<usize> {
        if !self.is_active() {
            return Err(AlienError::DOMAINCRASH);
        }
        self.domain.trap_frame_phy_addr()
    }

    fn heap_info(&self, tmp_heap_info: RRef<TmpHeapInfo>) -> AlienResult<RRef<TmpHeapInfo>> {
        if !self.is_active() {
            return Err(AlienError::DOMAINCRASH);
        }
        self.domain.heap_info(tmp_heap_info)
    }

    fn brk(&self, addr: usize) -> AlienResult<isize> {
        if !self.is_active() {
            return Err(AlienError::DOMAINCRASH);
        }
        self.domain.brk(addr)
    }

    fn get_fd(&self, fd: usize) -> AlienResult<InodeId> {
        if !self.is_active() {
            return Err(AlienError::DOMAINCRASH);
        }
        self.domain.get_fd(fd)
    }
    fn copy_to_user(&self, src: *const u8, dst: *mut u8, len: usize) -> AlienResult<()> {
        if !self.is_active() {
            return Err(AlienError::DOMAINCRASH);
        }
        self.domain.copy_to_user(src, dst, len)
    }
    fn copy_from_user(&self, src: *const u8, dst: *mut u8, len: usize) -> AlienResult<()> {
        if !self.is_active() {
            return Err(AlienError::DOMAINCRASH);
        }
        self.domain.copy_from_user(src, dst, len)
    }
}

#[derive(Debug)]
pub struct SysCallDomainProxy {
    id: u64,
    domain: Arc<dyn SysCallDomain>,
}

impl SysCallDomainProxy {
    pub fn new(id: u64, domain: Arc<dyn SysCallDomain>) -> Self {
        Self { id, domain }
    }
}

impl Basic for SysCallDomainProxy {
    fn is_active(&self) -> bool {
        self.domain.is_active()
    }
}

impl SysCallDomain for SysCallDomainProxy {
    fn init(&self) -> AlienResult<()> {
        self.domain.init()
    }

    fn call(&self, syscall_id: usize, args: [usize; 6]) -> AlienResult<isize> {
        if !self.is_active() {
            return Err(AlienError::DOMAINCRASH);
        }
        self.domain.call(syscall_id, args)
    }
}
