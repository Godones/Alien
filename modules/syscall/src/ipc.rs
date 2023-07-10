// #define FUTEX_WAIT		0
// #define FUTEX_WAKE		1
// #define FUTEX_FD		2
// #define FUTEX_REQUEUE		3
// #define FUTEX_CMP_REQUEUE	4
// #define FUTEX_WAKE_OP		5
// #define FUTEX_LOCK_PI		6
// #define FUTEX_UNLOCK_PI		7
// #define FUTEX_TRYLOCK_PI	8
// #define FUTEX_WAIT_BITSET	9
// #define FUTEX_WAKE_BITSET	10
// #define FUTEX_WAIT_REQUEUE_PI	11
// #define FUTEX_CMP_REQUEUE_PI	12
//
// #define FUTEX_PRIVATE_FLAG	128
// #define FUTEX_CLOCK_REALTIME	256
// #define FUTEX_CMD_MASK		~(FUTEX_PRIVATE_FLAG | FUTEX_CLOCK_REALTIME)
//
// #define FUTEX_WAIT_PRIVATE	(FUTEX_WAIT | FUTEX_PRIVATE_FLAG)
// #define FUTEX_WAKE_PRIVATE	(FUTEX_WAKE | FUTEX_PRIVATE_FLAG)
// #define FUTEX_REQUEUE_PRIVATE	(FUTEX_REQUEUE | FUTEX_PRIVATE_FLAG)
// #define FUTEX_CMP_REQUEUE_PRIVATE (FUTEX_CMP_REQUEUE | FUTEX_PRIVATE_FLAG)
// #define FUTEX_WAKE_OP_PRIVATE	(FUTEX_WAKE_OP | FUTEX_PRIVATE_FLAG)
// #define FUTEX_LOCK_PI_PRIVATE	(FUTEX_LOCK_PI | FUTEX_PRIVATE_FLAG)
// #define FUTEX_UNLOCK_PI_PRIVATE	(FUTEX_UNLOCK_PI | FUTEX_PRIVATE_FLAG)
// #define FUTEX_TRYLOCK_PI_PRIVATE (FUTEX_TRYLOCK_PI | FUTEX_PRIVATE_FLAG)
// #define FUTEX_WAIT_BITSET_PRIVATE	(FUTEX_WAIT_BITSET | FUTEX_PRIVATE_FLAG)
// #define FUTEX_WAKE_BITSET_PRIVATE	(FUTEX_WAKE_BITSET | FUTEX_PRIVATE_FLAG)
// #define FUTEX_WAIT_REQUEUE_PI_PRIVATE	(FUTEX_WAIT_REQUEUE_PI | \
// FUTEX_PRIVATE_FLAG)
// #define FUTEX_CMP_REQUEUE_PI_PRIVATE	(FUTEX_CMP_REQUEUE_PI | \
// FUTEX_PRIVATE_FLAG)

use numeric_enum_macro::numeric_enum;

numeric_enum! {
    #[repr(u32)]
    #[derive(Eq, PartialEq, Debug, Copy, Clone)]
    pub enum FutexOp {
    FutexWait = 0,
    FutexWake = 1,
    FutexFd = 2,
    FutexRequeue = 3,
    FutexCmpRequeue = 4,
    FutexWakeOp = 5,
    FutexLockPi = 6,
    FutexUnlockPi = 7,
    FutexTrylockPi = 8,
    FutexWaitBitset = 9,
    FutexWakeBitset = 10,
    FutexWaitRequeuePi = 11,
    FutexCmpRequeuePi = 12,
    FutexWaitPrivate = 128 | FutexOp::FutexWait as u32,
    FutexWakePrivate = 128 | FutexOp::FutexWake as u32,
    FutexRequeuePrivate = 128 | FutexOp::FutexRequeue as u32,
    FutexCmpRequeuePrivate = 128 | FutexOp::FutexCmpRequeue as u32,
    FutexWakeOpPrivate = 128 | FutexOp::FutexWakeOp as u32,
    FutexLockPiPrivate = 128 | FutexOp::FutexLockPi as u32,
    FutexUnlockPiPrivate = 128 | FutexOp::FutexUnlockPi as u32,
    FutexTrylockPiPrivate = 128 | FutexOp::FutexTrylockPi as u32,
    FutexWaitBitsetPrivate = 128 | FutexOp::FutexWaitBitset as u32,
    FutexWakeBitsetPrivate = 128 | FutexOp::FutexWakeBitset as u32,
    FutexWaitRequeuePiPrivate = 128 | FutexOp::FutexWaitRequeuePi as u32,
    FutexCmpRequeuePiPrivate = 128 | FutexOp::FutexCmpRequeuePi as u32,
}
}

#[derive(Clone, Copy, Debug)]
pub struct RobustList {
    pub head: usize,
    pub len: usize,
}

impl RobustList {
    // from strace
    pub const HEAD_SIZE: usize = 24;
}

impl Default for RobustList {
    fn default() -> Self {
        Self {
            head: 0,
            len: Self::HEAD_SIZE,
        }
    }
}
