#![no_std]
#![no_main]
extern crate alloc;

use alloc::{
    format,
    string::{String, ToString},
};

use Mstd::{
    domain::{register_domain, update_domain, DomainTypeRaw},
    fs::{close, open, OpenFlags},
};

type Result<T> = core::result::Result<T, String>;

#[derive(Clone)]
pub struct DomainHelperBuilder {
    ty: Option<DomainTypeRaw>,
    domain_file_name: Option<String>,
    domain_file_path: Option<String>,
    domain_name: Option<String>,
}

impl DomainHelperBuilder {
    pub fn new() -> Self {
        Self {
            ty: None,
            domain_file_name: None,
            domain_name: None,
            domain_file_path: None,
        }
    }

    /// Set the domain type
    pub fn ty(mut self, ty: DomainTypeRaw) -> Self {
        self.ty = Some(ty);
        self
    }

    /// Set the domain file name which is used to register the domain
    pub fn domain_file_name(mut self, domain_file_name: &str) -> Self {
        self.domain_file_name = Some(domain_file_name.to_string());
        self
    }

    /// Set the domain name which will be updated
    pub fn domain_name(mut self, domain_name: &str) -> Self {
        self.domain_name = Some(domain_name.to_string());
        self
    }

    /// Set the domain file path which will be opened and registered
    pub fn domain_file_path(mut self, domain_file_path: &str) -> Self {
        self.domain_file_path = Some(domain_file_path.to_string());
        self
    }
}

impl DomainHelperBuilder {
    pub fn register_domain_file(self) -> Result<()> {
        let ty = self.ty.ok_or("Domain type is not set")?;
        let domain_file_name = self
            .domain_file_name
            .as_ref()
            .ok_or("Domain file name is not set")?;
        let domain_file_path = self
            .domain_file_path
            .as_ref()
            .ok_or("Domain file path is not set")?;
        let file = open(domain_file_path.as_str(), OpenFlags::O_RDONLY);
        if file < 0 {
            return Err(format!("Failed to open {}", domain_file_path));
        }
        let res = register_domain(file as _, ty, domain_file_name.as_str());
        if res != 0 {
            return Err(format!("Failed to register domain {}", domain_file_name));
        }
        close(file as _);
        Ok(())
    }
    pub fn update_domain(self) -> Result<()> {
        let ty = self.ty.ok_or("Domain type is not set")?;
        let domain_name = self.domain_name.as_ref().ok_or("Domain name is not set")?;
        let domain_file_name = self
            .domain_file_name
            .as_ref()
            .ok_or("Domain file name is not set")?;
        let res = update_domain(domain_name.as_str(), domain_file_name.as_str(), ty);
        if res != 0 {
            return Err(format!(
                "Failed to update domain {} using file {}",
                domain_name, domain_file_name
            ));
        }
        Ok(())
    }
}
