#![cfg_attr(not(test), no_std)]
extern crate alloc;
pub mod dentry;
pub mod error;
pub mod file;
pub mod fstype;
pub mod inode;
pub mod path;
pub mod superblock;
pub mod utils;
pub type VfsResult<T> = Result<T, error::VfsError>;

pub use rref::RRefVec;
