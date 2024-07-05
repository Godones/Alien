use alloc::{
    boxed::Box,
    collections::BTreeMap,
    string::{String, ToString},
    sync::Arc,
    vec::Vec,
};

use corelib::domain_info::DomainFileInfo;
use interface::*;
use ksync::RwLock;

use crate::{
    domain_helper::{alloc_domain_id, DomainCreate, DOMAIN_INFO},
    domain_loader::loader::DomainLoader,
    domain_proxy::*,
    error::{AlienError, AlienResult},
    register_domain,
};

static DOMAIN_ELF: RwLock<BTreeMap<String, DomainData>> = RwLock::new(BTreeMap::new());

#[derive(Clone)]
struct DomainData {
    ty: DomainTypeRaw,
    data: Arc<Vec<u8>>,
}

/// Register the domain elf data with the given identifier.
pub fn register_domain_elf(domain_file_name: &str, elf: Vec<u8>, ty: DomainTypeRaw) {
    let elf_len = elf.len();
    let mut binding = DOMAIN_ELF.write();

    if binding.iter().find(|(k, f)| {
        k == &domain_file_name && elf.len() == f.data.len()
    }).is_some(){
        println!("Domain {} already registered", domain_file_name);
        return;
    }
    platform::println!("<register domain>: {}", domain_file_name);
    binding.insert(
        domain_file_name.to_string(),
        DomainData {
            ty,
            data: Arc::new(elf),
        },
    );
    // update domain info
    let mut domain_info_lock = DOMAIN_INFO.lock();
    let file_info = DomainFileInfo::new(domain_file_name.to_string(), elf_len);
    domain_info_lock
        .ty_list
        .entry(ty)
        .or_insert_with(Vec::new)
        .push(file_info);
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
    ($proxy_name:ident,$ty:expr, $domain_file_name:expr, $data:expr) => {
        crate::domain_loader::creator::create_domain_special::<$proxy_name, _>(
            $ty,
            $domain_file_name,
            $data,
            None,
        )
    };
    ($proxy_name:ident,$ty:expr, $domain_file_name:expr) => {
        crate::domain_loader::creator::create_domain_special::<$proxy_name, _>(
            $ty,
            $domain_file_name,
            None,
            None,
        )
    };
    ($proxy_name:ident,$ty:expr, $domain_file_name:expr, $data:expr, $use_old_id:expr) => {
        crate::domain_loader::creator::create_domain_special::<$proxy_name, _>(
            $ty,
            $domain_file_name,
            $data,
            $use_old_id,
        )
    };
}

pub fn create_domain_special<P, T>(
    ty: DomainTypeRaw,
    domain_file_name: &str,
    data: Option<Vec<u8>>,
    use_old_id: Option<u64>,
) -> AlienResult<(Arc<P>, DomainFileInfo)>
where
    P: ProxyBuilder<T = Box<T>>,
    T: ?Sized,
{
    let res = create_domain(ty, domain_file_name, data, use_old_id)
        .map(|(_id, domain, loader)| {
            let file_info = loader.domain_file_info();
            (Arc::new(P::build(domain, loader)), file_info)
        })
        .unwrap_or_else(|| {
            println!("Create empty domain: {}", domain_file_name);
            let loader = DomainLoader::empty();
            let file_info = loader.domain_file_info();
            let res = Arc::new(P::build_empty(loader));
            (res, file_info)
        });
    Ok(res)
}

pub struct DomainCreateImpl;
impl DomainCreate for DomainCreateImpl {
    fn create_domain(
        &self,
        domain_file_name: &str,
        identifier: &mut [u8],
    ) -> AlienResult<DomainType> {
        match domain_file_name {
            "fatfs" | "ramfs" => {
                let (fs, domain_file_info) =
                    create_domain!(FsDomainProxy, DomainTypeRaw::FsDomain, domain_file_name)?;
                fs.init_by_box(Box::new(())).unwrap();
                let identifier_res = register_domain!(
                    domain_file_name,
                    domain_file_info,
                    DomainType::FsDomain(fs.clone()),
                    false
                );
                let min_len = identifier.len().min(identifier_res.len());
                identifier[..min_len].copy_from_slice(&identifier_res.as_bytes()[..min_len]);
                Ok(DomainType::FsDomain(fs))
            }
            _ => Err(AlienError::ENOSYS),
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

    Some((id, domain, domain_loader))
}
