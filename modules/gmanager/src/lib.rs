#![no_std]

extern crate alloc;
#[cfg(test)]
extern crate std;
use alloc::vec::Vec;

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
        let data = self
            .data
            .iter()
            .enumerate()
            .find(|(_index, fd)| fd.is_none());
        match data {
            Some(_) => self.usable = data.unwrap().0,
            None => self.usable = self.data.len(),
        }
        Ok(ans)
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
        let val = self.data.get(index).unwrap();
        Ok(val.clone())
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
    use crate::MinimalManager;
    use std::println;

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
        let ans = manager.remove(1).unwrap();
        let index = manager.insert(10).unwrap();
        assert_eq!(index, 1);
        println!("gmanager test passed");
    }
}
