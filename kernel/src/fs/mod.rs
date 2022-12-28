//! use fat32

mod stdio;
mod vfs;

use gmanager::MinimalManager;
pub use stdio::*;

pub use vfs::fs_repl;

pub trait File: Send + Sync {
    fn write(&self, buf: &[u8]) -> usize;
    fn read(&self, buf: &mut [u8]) -> usize;
}

#[allow(unused)]
pub fn test_gmanager() {
    let mut manager = MinimalManager::<usize>::new(10);
    for i in 0..10 {
        let index = manager.insert(10).unwrap();
        assert_eq!(index, i);
    }
    let index = manager.insert(10);
    assert!(index.is_err());
    let ans = manager.remove(10);
    assert!(ans.is_err());
    let ans = manager.remove(1).unwrap();
    let index = manager.insert(10).unwrap();
    assert_eq!(index, 1);
    let index = manager.insert(10);
    assert!(index.is_err());

    println!("gmanager test passed");
}
