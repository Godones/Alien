use alloc::vec::Vec;

pub use action::{
    SigAction, SigActionDefault, SigActionFlags, SIGNAL_RETURN_TRAP, SIG_DFL, SIG_IGN,
};
pub use number::SignalNumber;
pub use siginfo::{SigInfo, SigProcMaskHow};
pub use ucontext::SignalUserContext;

mod action;
mod number;
mod siginfo;
mod ucontext;

/// signal 中用到的 bitset 长度。
pub const SIGSET_SIZE_IN_BYTE: usize = 8;
/// 所有可能的信号数。有多少可能的信号，内核就要为其保存多少个 SigAction
pub const SIGSET_SIZE_IN_BIT: usize = SIGSET_SIZE_IN_BYTE * 8; // =64

/// 处理信号的结构，每个线程有一个，根据 clone 的参数有可能是共享的
#[derive(Clone, Copy, Debug)]
pub struct SignalHandlers {
    /// 所有的处理函数
    actions: [Option<SigAction>; SIGSET_SIZE_IN_BIT],
}

impl SignalHandlers {
    /// 新建一个信号模块
    pub fn new() -> Self {
        Self {
            actions: [None; SIGSET_SIZE_IN_BIT],
        }
    }
    /// 清空模块。
    /// exec时需要将信号模块恢复为默认。
    pub fn clear(&mut self) {
        for action in &mut self.actions {
            action.take();
        }
    }
    /// 获取某个信号对应的 SigAction。
    /// 因为 signum 的范围是 \[1,64\]，所以要 -1
    pub fn get_action<'a>(&self, signum: usize, action_pos: &mut SigAction) {
        if let Some(action) = self.actions[signum - 1] {
            *action_pos = action;
        }
    }
    /// 获取某个信号对应的 SigAction，如果存在，则返回其引用
    /// 因为 signum 的范围是 \[1,64\]，所以要 -1
    pub fn get_action_ref(&self, signum: usize) -> Option<&SigAction> {
        if self.actions[signum - 1].is_some()
            && self.actions[signum - 1].unwrap().handler == SIG_DFL
        {
            None
        } else {
            self.actions[signum - 1].as_ref()
        }
        //if signum != 33 {&self.actions[signum - 1]} else {&None}
    }
    /// 修改某个信号对应的 SigAction。
    /// 因为 signum 的范围是 \[1,64\]，所以内部要 -1
    pub fn set_action(&mut self, signum: usize, action_pos: &SigAction) {
        self.actions[signum - 1] = Some(*action_pos);
        //self.actions[signum - 1].as_mut().unwrap().flags |= SigActionFlags::SA_SIGINFO;
    }
}

/// 接受信号的结构，每个线程都独有，不会共享
#[derive(Clone, Copy, Debug)]
pub struct SignalReceivers {
    /// 掩码，表示哪些信号是当前线程不处理的。（目前放在进程中，实现了线程之后每个线程应该各自有一个）
    pub mask: SimpleBitSet,
    /// 当前已受到的信号
    pub sig_received: SimpleBitSet,
}

impl SignalReceivers {
    /// 新建一个处理模块
    pub fn new() -> Self {
        Self {
            mask: SimpleBitSet::default(),
            sig_received: SimpleBitSet::default(),
        }
    }
    /// 清空模块。
    pub fn clear(&mut self) {
        self.mask = SimpleBitSet::default();
        self.sig_received = SimpleBitSet::default();
    }
    /// 处理一个信号。如果有收到的信号，则返回信号编号。否则返回 None
    pub fn get_one_signal(&mut self) -> Option<usize> {
        self.sig_received.find_first_one(self.mask).map(|pos| {
            self.sig_received.remove_bit(pos);
            pos + 1
        })
    }

    pub fn have_signal(&self) -> bool {
        self.sig_received.find_first_one(self.mask).is_some()
    }

    pub fn have_signal_with_number(&self) -> Option<usize> {
        // self.sig_received.find_first_one(self.mask)
        self.sig_received
            .find_first_one_without_mask()
            .map(|n| n + 1)
    }

    pub fn check_signal(&mut self, signum: usize) -> bool {
        self.sig_received.check_bit(signum - 1)
    }

    /// 尝试添加一个 bit 作为信号。发送的信号如果在 mask 中，则仍然会发送，只是可能不触发
    /// 因为 signum 的范围是 \[1,64\]，所以内部要 -1
    ///
    /// 因为没有要求判断信号是否发送成功的要求，所有这里不设返回值
    pub fn try_add_bit(&mut self, signum: usize) {
        //info!("try add {}, mask = {:x}", signum, self.mask.0);
        self.sig_received.add_bit(signum - 1);
    }
}

#[derive(Debug, Default, Copy, Clone)]
pub struct SimpleBitSet(pub usize);

impl SimpleBitSet {
    /// 寻找不在mask中的最小的 1 的位置，如果有，返回其位置，如没有则返回 None。
    pub fn find_first_one(&self, mask: SimpleBitSet) -> Option<usize> {
        let ans = (self.0 & !mask.0).trailing_zeros() as usize;
        if ans == 64 {
            None
        } else {
            Some(ans)
        }
    }

    pub fn find_first_one_without_mask(&self) -> Option<usize> {
        let ans = self.0.trailing_zeros() as usize;
        if ans == 64 {
            None
        } else {
            Some(ans)
        }
    }

    pub fn bits(&self) -> usize {
        self.0
    }

    pub fn remove_bit(&mut self, pos: usize) {
        self.0 &= !(1 << pos);
    }

    pub fn add_bit(&mut self, pos: usize) {
        self.0 |= 1 << pos;
    }

    pub fn check_bit(&mut self, pos: usize) -> bool {
        self.0 & (1 << pos) != 0
    }
}

impl Into<Vec<SignalNumber>> for SimpleBitSet {
    fn into(self) -> Vec<SignalNumber> {
        let mut ans = Vec::new();
        for i in 0..64 {
            if self.0 & (1 << i) != 0 {
                ans.push(SignalNumber::try_from(i + 1).unwrap());
            }
        }
        ans
    }
}

impl From<usize> for SimpleBitSet {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl core::ops::Sub for SimpleBitSet {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 & !rhs.0)
    }
}

impl core::ops::SubAssign for SimpleBitSet {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 &= !rhs.0;
    }
}

impl core::ops::Add for SimpleBitSet {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl core::ops::AddAssign for SimpleBitSet {
    fn add_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}
