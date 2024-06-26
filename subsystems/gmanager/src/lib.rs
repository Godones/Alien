//! A minimal index manager
//!
//! # Example
//! ```
//! use gmanager::MinimalManager;
//! let mut manager = MinimalManager::<usize>::new(10);
//! for i in 0..10 {
//!     let index = manager.insert(10).unwrap();
//!     assert_eq!(index, i);
//!  }
//! let index = manager.insert(10);
//! assert!(index.is_err());
//! let ans = manager.remove(10);
//! assert!(ans.is_err());
//! let _ans = manager.remove(1).unwrap();
//! let index = manager.insert(10).unwrap();
//! assert_eq!(index, 1);
//! ```
#![cfg_attr(not(test), no_std)]

extern crate alloc;

use alloc::vec::Vec;

/// A minimal index manager
#[derive(Debug, Clone)]
pub struct MinimalManager<T: Clone> {
    data: Vec<Option<T>>,
    // 记录最小可用索引
    usable: usize,
    // 最大可用
    max: usize,
}

impl<T: Clone> MinimalManager<T> {
    pub fn new(max: usize) -> MinimalManager<T> {
        Self {
            data: Vec::new(),
            usable: 0,
            max,
        }
    }
    /// reset the max index
    pub fn set_max(&mut self, val: usize) -> bool {
        if val > self.max {
            self.max = val;
            return true;
        }
        // check [val:max] has data
        for i in val..self.max {
            let data = self.data.get(i);
            if data.is_some() && data.unwrap().is_some() {
                // delete the data
                self.remove(i).unwrap();
            }
        }
        self.max = val;
        true
    }

    /// get the max index
    pub fn max(&self) -> usize {
        self.max
    }

    /// insert a value
    pub fn insert(&mut self, val: T) -> Result<usize, ManagerError> {
        if self.usable == self.max {
            return Err(ManagerError::NoSpace);
        }
        if self.data.len() <= self.usable {
            self.data.push(None);
        }
        self.data[self.usable] = Some(val);
        let ans = self.usable;
        //查找下一个可用的位置
        self.usable = self.find_next_index().unwrap_or(self.data.len());
        Ok(ans)
    }

    /// find the next usable index
    fn find_next_index(&self) -> Option<usize> {
        let data = self
            .data
            .iter()
            .enumerate()
            .find(|(_index, fd)| fd.is_none())
            .map(|x| x.0);
        data
    }

    /// remove a value by index
    pub fn remove(&mut self, index: usize) -> Result<(), ManagerError> {
        if index >= self.max {
            return Err(ManagerError::IndexOver);
        }
        let val = self.data.get(index).unwrap();
        if val.is_none() {
            return Err(ManagerError::NotExist);
        }
        self.data[index] = None;
        if self.usable > index {
            self.usable = index;
        }
        Ok(())
    }

    /// get a value by index
    pub fn get(&self, index: usize) -> Result<Option<T>, ManagerError> {
        if index > self.max {
            return Err(ManagerError::IndexOver);
        }
        let val = self.data.get(index);
        if val.is_none() {
            return Err(ManagerError::NotExist);
        }
        let val = val.unwrap();
        Ok(val.clone())
    }

    /// User should ensure that the index is valid
    pub fn insert_with_index(&mut self, index: usize, val: T) -> Result<(), ManagerError> {
        if index >= self.max {
            return Err(ManagerError::IndexOver);
        }
        if index >= self.data.len() {
            self.data.resize(index + 1, None);
        }
        self.data[index] = Some(val);
        if index == self.usable {
            self.usable = self.find_next_index().unwrap_or(self.data.len());
        }
        Ok(())
    }

    /// clear all data
    pub fn clear(&mut self) -> Vec<T> {
        let res = self
            .data
            .iter()
            .filter(|x| x.is_some())
            .map(|x| x.clone().unwrap())
            .collect();
        self.data.clear();
        self.usable = 0;
        res
    }
}

/// Error type
#[derive(Debug)]
pub enum ManagerError {
    NoSpace = 0,
    NotExist = 1,
    IndexOver = 2,
}

impl From<usize> for ManagerError {
    fn from(value: usize) -> Self {
        match value {
            0 => ManagerError::NoSpace,
            1 => ManagerError::NotExist,
            2 => ManagerError::IndexOver,
            _ => panic!("Unknown error code"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::MinimalManager;
    #[test]
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
        let _ans = manager.remove(1).unwrap();
        let index = manager.insert(10).unwrap();
        assert_eq!(index, 1);
    }
}
