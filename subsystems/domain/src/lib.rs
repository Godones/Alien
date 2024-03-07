#![no_std]

extern crate alloc;

use alloc::sync::Arc;
use domain_helper::alloc_domain_id;
use domain_loader::DomainLoader;
use interface::{BlkDeviceDomain, CacheBlkDeviceDomain, FsDomain, RtcDomain, VfsDomain};
use log::info;
use proxy::{BlkDomainProxy, CacheBlkDomainProxy, FsDomainProxy, RtcDomainProxy, VfsDomainProxy};

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
static BLK_DOMAIN: &'static [u8] = include_bytes_align_as!(usize, "../../../build/gblk_domain.bin");
static FATFS_DOMAIN: &'static [u8] =
    include_bytes_align_as!(usize, "../../../build/gfatfs_domain.bin");
static RTC_DOMAIN: &'static [u8] =
    include_bytes_align_as!(usize, "../../../build/ggoldfish_domain.bin");
static VFS_DOMAIN: &'static [u8] = include_bytes_align_as!(usize, "../../../build/gvfs_domain.bin");
static CACHE_BLK_DOMAIN: &'static [u8] =
    include_bytes_align_as!(usize, "../../../build/gcache_blk_domain.bin");

static SHADOW_BLK_DOMAIN: &'static [u8] =
    include_bytes_align_as!(usize, "../../../build/gshadow_blk_domain.bin");

fn fatfs_domain() -> Arc<dyn FsDomain> {
    let mut domain = DomainLoader::new(FATFS_DOMAIN);
    domain.load().unwrap();
    let id = alloc_domain_id();
    let fatfs = domain.call(id);
    Arc::new(FsDomainProxy::new(id, fatfs))
}

fn blk_domain() -> Arc<dyn BlkDeviceDomain> {
    let mut domain = DomainLoader::new(BLK_DOMAIN);
    domain.load().unwrap();
    let id = alloc_domain_id();
    let dev = domain.call::<dyn BlkDeviceDomain>(id);
    info!(
        "dev capacity: {:?}MB",
        dev.get_capacity().unwrap() / 1024 / 1024
    );
    Arc::new(BlkDomainProxy::new(id, dev, domain))
}

fn rtc_domain() -> Arc<dyn RtcDomain> {
    let mut domain = DomainLoader::new(RTC_DOMAIN);
    domain.load().unwrap();
    let id = alloc_domain_id();
    let rtc = domain.call(id);
    Arc::new(RtcDomainProxy::new(id, rtc))
}

fn vfs_domain() -> Arc<dyn VfsDomain> {
    let mut domain = DomainLoader::new(VFS_DOMAIN);
    domain.load().unwrap();
    let id = alloc_domain_id();
    let vfs = domain.call(id);
    Arc::new(VfsDomainProxy::new(id, vfs))
}

fn cache_blk_domain() -> Arc<dyn CacheBlkDeviceDomain> {
    let mut domain = DomainLoader::new(CACHE_BLK_DOMAIN);
    domain.load().unwrap();
    let id = alloc_domain_id();
    let cache_blk = domain.call(id);
    Arc::new(CacheBlkDomainProxy::new(id, cache_blk))
}

fn shadow_blk_domain() -> Arc<dyn BlkDeviceDomain> {
    let mut domain = DomainLoader::new(SHADOW_BLK_DOMAIN);
    domain.load().unwrap();
    let id = alloc_domain_id();
    let shadow_blk = domain.call(id);
    Arc::new(BlkDomainProxy::new(id, shadow_blk, domain))
}

pub fn load_domains() {
    info!("Load blk domain, size: {}KB", BLK_DOMAIN.len() / 1024);
    let dev = blk_domain();
    domain_helper::register_domain("blk", dev);
    // info!("Load fatfs domain, size: {}KB", FATFS_DOMAIN.len() / 1024);
    // let fs = fatfs_domain();
    // domain_helper::register_domain("fatfs", fs);

    info!(
        "Load shadow blk domain, size: {}KB",
        SHADOW_BLK_DOMAIN.len() / 1024
    );
    let shadow_blk = shadow_blk_domain();
    domain_helper::register_domain("shadow_blk", shadow_blk);

    info!("Load rtc domain, size: {}KB", RTC_DOMAIN.len() / 1024);
    let rtc = rtc_domain();
    domain_helper::register_domain("rtc", rtc);
    info!(
        "Load cache blk domain, size: {}KB",
        CACHE_BLK_DOMAIN.len() / 1024
    );
    let cache_blk = cache_blk_domain();
    domain_helper::register_domain("cache_blk", cache_blk);
    info!("Load vfs domain, size: {}KB", VFS_DOMAIN.len() / 1024);
    let vfs = vfs_domain();
    domain_helper::register_domain("vfs", vfs);
    platform::println!("Load domains done");
}
