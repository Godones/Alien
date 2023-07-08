#![cfg_attr(not(test), no_std)]

extern crate alloc;

use alloc::vec::Vec;

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
    pub fn set_max(&mut self, val: usize) -> bool {
        if val > self.max {
            self.max = val;
            return true;
        }
        // check [val:max] has data
        for i in val..self.max {
            let data = self.data.get(i);
            if data.is_some() && data.unwrap().is_some() {
                return false;
            }
        }
        self.max = val;
        true
    }

    pub fn max(&self) -> usize {
        self.max
    }

    pub fn data(&self) -> &Vec<Option<T>> {
        &self.data
    }
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

    pub fn find_next_index(&self) -> Option<usize> {
        let data = self
            .data
            .iter()
            .enumerate()
            .find(|(_index, fd)| fd.is_none())
            .map(|x| x.0);
        data
    }
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
    #[allow(unused)]
    pub fn is_usable(&self, index: usize) -> Result<bool, ManagerError> {
        if index > self.max {
            return Err(ManagerError::IndexOver);
        }
        let val = self.data.get(index).unwrap();
        Ok(val.is_none())
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
}

#[derive(Debug)]
pub enum ManagerError {
    NoSpace,
    NotExist,
    IndexOver,
}

#[cfg(test)]
mod tests {
    use std::println;

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
        println!("gmanager test passed");
    }
}
