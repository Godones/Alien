#![no_std]

mod loader;

extern crate alloc;
use crate::loader::DomainLoader;
use alloc::boxed::Box;
use alloc::sync::Arc;
use domain_helper::{alloc_domain_id, DomainSyscall, SharedHeapAllocator};
use interface::{BlkDeviceDomain, FsDomain};
use libsyscall::Syscall;
use log::info;
use proxy::{BlkDomainProxy, FsDomainProxy};
use rref::SharedHeap;

#[macro_use]
mod macros {
    #[repr(C)] // guarantee 'bytes' comes after '_align'
    pub struct AlignedAs<Align, Bytes: ?Sized> {
        pub _align: [Align; 0],
        pub bytes: Bytes,
    }

    macro_rules! include_bytes_align_as {
        ($align_ty:ty, $path:literal) => {{
            // const block expression to encapsulate the static
            use $crate::macros::AlignedAs;

            // this assignment is made possible by CoerceUnsized
            static ALIGNED: &AlignedAs<$align_ty, [u8]> = &AlignedAs {
                _align: [],
                bytes: *include_bytes!($path),
            };

            &ALIGNED.bytes
        }};
    }
}

macro_rules! def_func {
    ($ret:ty,$( $ty:ty ),*) => {
        fn(Box<dyn Syscall>,u64, Box<dyn SharedHeap>,$( $ty ),*) -> $ret
    };
}

static BLK_DOMAIN: &'static [u8] = include_bytes_align_as!(usize, "../../../build/blk_domain.bin");
static FATFS_DOMAIN: &'static [u8] =
    include_bytes_align_as!(usize, "../../../build/fatfs_domain.bin");

fn fatfs_domain() -> Arc<dyn FsDomain> {
    type F = def_func!(Arc<dyn FsDomain>,);
    let mut domain = DomainLoader::new();
    domain.load(FATFS_DOMAIN).unwrap();
    let main = unsafe { core::mem::transmute::<*const (), F>(domain.entry() as *const ()) };
    let id = alloc_domain_id();
    let fatfs = main(Box::new(DomainSyscall), id, Box::new(SharedHeapAllocator));
    Arc::new(FsDomainProxy::new(id, fatfs))
}

fn blk_domain() -> Arc<dyn BlkDeviceDomain> {
    type F = def_func!(Arc<dyn BlkDeviceDomain>, usize);
    let mut domain = DomainLoader::new();
    domain.load(BLK_DOMAIN).unwrap();
    let main = unsafe { core::mem::transmute::<*const (), F>(domain.entry() as *const ()) };
    let id = alloc_domain_id();
    let dev = main(
        Box::new(DomainSyscall),
        id,
        Box::new(SharedHeapAllocator),
        0x10008000,
    );
    info!(
        "dev capacity: {:?}MB",
        dev.get_capacity().unwrap() / 1024 / 1024
    );
    Arc::new(BlkDomainProxy::new(id, dev))
}

pub fn load_domains() {
    info!("Load blk domain, size: {}KB", BLK_DOMAIN.len() / 1024);
    let dev = blk_domain();
    domain_helper::register_domain("blk", dev);
    info!("Load fatfs domain, size: {}KB", FATFS_DOMAIN.len() / 1024);
    let fs = fatfs_domain();
    domain_helper::register_domain("fatfs", fs);
    platform::println!("Load domains done");
}
