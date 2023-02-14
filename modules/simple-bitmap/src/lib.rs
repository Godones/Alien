#![no_std]


extern crate alloc;
#[cfg(test)]
extern crate std;

// 128
pub struct Bitmap<const N: usize> {
    data: [u8; N],
}

impl<const N: usize> Default for Bitmap<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> Bitmap<N> {
    pub fn new() -> Self {
        Self { data: [0; N] }
    }
    pub fn set(&mut self, index: usize) {
        let (byte_index, bit_index) = (index / 8, index % 8);
        self.data[byte_index] |= 1 << bit_index;
    }

    pub fn clear(&mut self, index: usize) {
        let (byte_index, bit_index) = (index / 8, index % 8);
        self.data[byte_index] &= !(1 << bit_index);
    }

    /// 测试位图中的某一位是否未被使用
    fn test(&self, index: usize) -> bool {
        let (byte_index, bit_index) = (index / 8, index % 8);
        self.data[byte_index] & (1 << bit_index) != 0
    }
    fn find_first_zero(&self) -> Option<usize> {
        for (i, byte) in self.data.iter().enumerate() {
            if *byte != 0xff {
                for j in 0..8 {
                    if *byte & (1 << j) == 0 {
                        return Some(i * 8 + j);
                    }
                }
            }
        }
        None
    }
    pub fn dealloc(&mut self, index: usize) {
        self.clear(index);
    }
    pub fn alloc(&mut self) -> Option<usize> {
        let index = self.find_first_zero()?;
        self.set(index);
        Some(index)
    }
    /// 分配连续 bit
    pub fn alloc_contiguous(&mut self, count: usize, _align_log2: usize) -> Option<usize> {
        let mut index = self.find_first_zero()?;
        while index < N * 8 {
            let end = index + count;
            if end > N * 8 {
                return None;
            }
            let mut ok = false;
            for i in index..end {
                if self.test(i) {
                    index = i + 1;
                    ok = false;
                    break;
                } else {
                    ok = true;
                }
            }
            if ok {
                for i in index..end {
                    self.set(i);
                }
                return Some(index);
            }
        }
        None
    }
}

#[cfg(test)]
mod test {
    use crate::Bitmap;
    #[allow(unused)]
    #[test]
    fn test_alloc() {
        let mut bitmap = Bitmap::<16>::new();
        assert_eq!(bitmap.alloc(), Some(0));
        bitmap.set(1);
        assert_eq!(bitmap.alloc(), Some(2));
        let x = bitmap.alloc_contiguous(3, 0);
        assert_eq!(x, Some(3));
        let x = bitmap.alloc_contiguous(3, 0);
        assert_eq!(x, Some(6));
        bitmap.dealloc(7);
        let x = bitmap.alloc_contiguous(3, 0);
        assert_eq!(x, Some(9));
    }
}
