use alloc::{
    boxed::Box,
    collections::BTreeMap,
    string::{String, ToString},
    sync::Arc,
    vec::Vec,
};

use interface::*;
use ksync::RwLock;

use crate::{
    domain_helper,
    domain_helper::{alloc_domain_id, DomainCreate},
    domain_loader::loader::DomainLoader,
    domain_proxy::*,
    error::AlienResult,
};

static DOMAIN_ELF: RwLock<BTreeMap<String, DomainData>> = RwLock::new(BTreeMap::new());

#[derive(Clone)]
struct DomainData {
    ty: DomainTypeRaw,
    data: Arc<Vec<u8>>,
}

/// Register the domain elf data with the given identifier.
pub fn register_domain_elf(domain_file_name: &str, elf: Vec<u8>, ty: DomainTypeRaw) {
    let mut binding = DOMAIN_ELF.write();
    platform::println!("<register domain>: {}", domain_file_name);
    binding.insert(
        domain_file_name.to_string(),
        DomainData {
            ty,
            data: Arc::new(elf),
        },
    );
}

/// Unregister the domain elf data with the given identifier.
#[allow(unused)]
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
        crate::domain_loader::creator::create_domain_special::<$proxy_name, _>(
            $ty, $ident, $data, None,
        )
    };
    ($proxy_name:ident,$ty:expr, $ident:expr) => {
        crate::domain_loader::creator::create_domain_special::<$proxy_name, _>(
            $ty, $ident, None, None,
        )
    };
    ($proxy_name:ident,$ty:expr, $ident:expr, $data:expr, $use_old_id:expr) => {
        crate::domain_loader::creator::create_domain_special::<$proxy_name, _>(
            $ty,
            $ident,
            $data,
            $use_old_id,
        )
    };
}

pub fn create_domain_special<P, T>(
    ty: DomainTypeRaw,
    ident: &str,
    data: Option<Vec<u8>>,
    use_old_id: Option<u64>,
) -> AlienResult<Arc<P>>
where
    P: ProxyBuilder<T = Box<T>>,
    T: ?Sized,
{
    let res = create_domain(ty, ident, data, use_old_id)
        .map(|(_id, domain, loader)| Arc::new(P::build(domain, loader)))
        .unwrap_or_else(|| {
            println!("Create empty domain: {}", ident);
            let loader = DomainLoader::empty();
            let res = Arc::new(P::build_empty(loader));
            res
        });
    Ok(res)
}

pub struct DomainCreateImpl;
impl DomainCreate for DomainCreateImpl {
    fn create_domain(&self, domain_file_name: &str) -> Option<DomainType> {
        match domain_file_name {
            "fatfs" => {
                let fatfs = create_domain!(FsDomainProxy, DomainTypeRaw::FsDomain, "fatfs").ok()?;
                fatfs.init_by_box(Box::new(())).unwrap();
                domain_helper::register_domain(
                    domain_file_name,
                    DomainType::FsDomain(fatfs.clone()),
                    false,
                );
                Some(DomainType::FsDomain(fatfs))
            }
            "ramfs" => {
                let ramfs = create_domain!(FsDomainProxy, DomainTypeRaw::FsDomain, "ramfs").ok()?;
                ramfs.init_by_box(Box::new(())).unwrap();
                domain_helper::register_domain(
                    domain_file_name,
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
    domain_file_name: &str,
    elf: Option<Vec<u8>>,
    use_old_id: Option<u64>,
) -> Option<(u64, Box<T>, DomainLoader)> {
    match elf {
        Some(data) => {
            register_domain_elf(domain_file_name, data, ty);
        }
        None => {}
    }
    let data = DOMAIN_ELF.read().get(domain_file_name)?.clone();
    if data.ty != ty {
        return None;
    }
    info!("Load {:?} domain, size: {}KB", ty, data.data.len() / 1024);
    let mut domain_loader = DomainLoader::new(data.data, domain_file_name);
    domain_loader.load().unwrap();
    let id = alloc_domain_id();
    let domain = domain_loader.call(id, use_old_id);
    if let Some(use_old_id) = use_old_id {
        domain_helper::move_domain_database(use_old_id, id);
    }
    Some((id, domain, domain_loader))
}
