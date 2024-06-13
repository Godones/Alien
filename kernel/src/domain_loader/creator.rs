use alloc::{
    boxed::Box,
    collections::BTreeMap,
    string::{String, ToString},
    sync::Arc,
    vec::Vec,
};

use interface::*;
use ksync::RwLock;
use spin::Lazy;

use crate::{
    domain_helper,
    domain_helper::{alloc_domain_id, DomainCreate},
    domain_loader::loader::DomainLoader,
    domain_proxy::*,
    error::AlienResult,
};

static DOMAIN_ELF: Lazy<RwLock<BTreeMap<String, DomainData>>> =
    Lazy::new(|| RwLock::new(BTreeMap::new()));

#[derive(Clone)]
struct DomainData {
    ty: DomainTypeRaw,
    data: Arc<Vec<u8>>,
}

pub fn register_domain_elf(identifier: &str, elf: Vec<u8>, ty: DomainTypeRaw) {
    let mut binding = DOMAIN_ELF.write();
    platform::println!("<register domain>: {}", identifier);
    binding.insert(
        identifier.to_string(),
        DomainData {
            ty,
            data: Arc::new(elf),
        },
    );
}

pub fn unregister_domain_elf(identifier: &str) {
    let mut binding = DOMAIN_ELF.write();
    binding.remove(identifier);
}

#[macro_export]
/// Create a domain with the given proxy name, type, identifier, and optional data.
///
/// It will expand to `create_domain_special::<$proxy_name, _>($ty, $ident, $data)`.
macro_rules! create_domain {
    ($proxy_name:ident,$ty:expr, $ident:expr, $data:expr) => {
        crate::domain_loader::creator::create_domain_special::<$proxy_name, _>($ty, $ident, $data)
    };
    ($proxy_name:ident,$ty:expr, $ident:expr) => {
        crate::domain_loader::creator::create_domain_special::<$proxy_name, _>($ty, $ident, None)
    };
}

pub fn create_domain_special<P, T>(
    ty: DomainTypeRaw,
    ident: &str,
    data: Option<Vec<u8>>,
) -> AlienResult<Arc<P>>
where
    P: ProxyBuilder<T = Box<T>>,
    T: ?Sized,
{
    let res = create_domain(ty, ident, data)
        .map(|(id, domain, loader)| Arc::new(P::build(id, domain, loader)))
        .unwrap_or_else(|| {
            println!("Create empty domain: {}", ident);
            let id = alloc_domain_id();
            let loader = DomainLoader::empty();
            let res = Arc::new(P::build_empty(id, loader));
            res
        });
    Ok(res)
}

pub struct DomainCreateImpl;
impl DomainCreate for DomainCreateImpl {
    fn create_domain(&self, identifier: &str) -> Option<DomainType> {
        match identifier {
            "fatfs" => {
                let fatfs = create_domain!(FsDomainProxy, DomainTypeRaw::FsDomain, "fatfs").ok()?;
                fatfs.init_by_box(Box::new(())).unwrap();
                domain_helper::register_domain(
                    identifier,
                    DomainType::FsDomain(fatfs.clone()),
                    false,
                );
                Some(DomainType::FsDomain(fatfs))
            }
            "ramfs" => {
                let ramfs = create_domain!(FsDomainProxy, DomainTypeRaw::FsDomain, "ramfs").ok()?;
                ramfs.init_by_box(Box::new(())).unwrap();
                domain_helper::register_domain(
                    identifier,
                    DomainType::FsDomain(ramfs.clone()),
                    false,
                );
                Some(DomainType::FsDomain(ramfs))
            }
            _ => None,
        }
    }
}

pub fn create_domain<T: ?Sized>(
    ty: DomainTypeRaw,
    ident: &str,
    elf: Option<Vec<u8>>,
) -> Option<(u64, Box<T>, DomainLoader)> {
    match elf {
        Some(data) => {
            register_domain_elf(ident, data, ty);
        }
        None => {}
    }
    let data = DOMAIN_ELF.read().get(ident)?.clone();
    if data.ty != ty {
        return None;
    }
    info!("Load {:?} domain, size: {}KB", ty, data.data.len() / 1024);
    let mut domain_loader = DomainLoader::new(data.data, ident);
    domain_loader.load().unwrap();
    let id = alloc_domain_id();
    let domain = domain_loader.call(id);
    Some((id, domain, domain_loader))
}
