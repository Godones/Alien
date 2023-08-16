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

use bitflags::bitflags;
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
/*
 * SHMMNI, SHMMAX and SHMALL are default upper limits which can be
 * modified by sysctl. The SHMMAX and SHMALL values have been chosen to
 * be as large possible without facilitating scenarios where userspace
 * causes overflows when adjusting the limits via operations of the form
 * "retrieve current limit; add X; update limit". It is therefore not
 * advised to make SHMMAX and SHMALL any larger. These limits are
 * suitable for both 32 and 64-bit systems.
 */
// #define SHMMIN 1			 /* min shared seg size (bytes) */
// #define SHMMNI 4096			 /* max num of segs system wide */
// #define SHMMAX (ULONG_MAX - (1UL << 24)) /* max shared seg size (bytes) */
// #define SHMALL (ULONG_MAX - (1UL << 24)) /* max shm system wide (pages) */
// #define SHMSEG SHMMNI			 /* max shared segs per process */
pub const SHMMIN: usize = 1;
pub const SHMMNI: usize = 4096;
pub const SHMMAX: usize = usize::MAX - (1 << 24);
pub const SHMALL: usize = usize::MAX - (1 << 24);
pub const SHMSEG: usize = SHMMNI;

/*
 * shmget() shmflg values.
 */
/* The bottom nine bits are the same as open(2) mode flags */
// #define SHM_R		0400	/* or S_IRUGO from <linux/stat.h> */
// #define SHM_W		0200	/* or S_IWUGO from <linux/stat.h> */
// /* Bits 9 & 10 are IPC_CREAT and IPC_EXCL */
// #define SHM_HUGETLB	04000	/* segment will use huge TLB pages */
// #define SHM_NORESERVE	010000	/* don't check for reservations */
// #define IPC_CREAT  01000
// #define IPC_EXCL   02000

bitflags! {
    pub struct ShmGetFlags: i32 {
        /// 
        const SHM_R = 0o400;
        /// 
        const SHM_W = 0o200;
        /// Create a new segment. If this flag is not used, then shmget() will find the segment associated with key and check to see if the user has permission to access the segment.
        const IPC_CREAT = 0o1000;
        /// This flag is used with IPC_CREAT to ensure that this call creates the segment.  If the segment already exists, the call fails.
        const IPC_EXCL = 0o2000;
        /// segment will use huge TLB pages
        const SHM_HUGETLB = 0o4000;
        /// don't check for reservations
        const SHM_NORESERVE = 0o10000;
    }
}

/*
 * shmat() shmflg values
 */
// #define	SHM_RDONLY	010000	/* read-only access */
// #define	SHM_RND		020000	/* round attach address to SHMLBA boundary */
// #define	SHM_REMAP	040000	/* take-over region on attach */
// #define	SHM_EXEC	0100000	/* execution access */
bitflags! {
    pub struct ShmAtFlags: i32 {
        /// read-only access
        const SHM_RDONLY = 0o10000;
        /// round attach address to SHMLBA boundary
        const SHM_RND = 0o20000;
        /// take-over region on attach
        const SHM_REMAP = 0o40000;
        /// execution access
        const SHM_EXEC = 0o100000;
    }
}

// * Control commands for `msgctl', `semctl', and `shmctl'.  */
// #define IPC_RMID	0		/* remove identifier */
// #define IPC_SET		1		/* set `ipc_perm' options */
// #define IPC_STAT	2		/* get `ipc_perm' options */
/* super user shmctl commands */
// #define SHM_LOCK 	11
// #define SHM_UNLOCK 	12
//
// /* ipcs ctl commands */
// #define SHM_STAT	13
// #define SHM_INFO	14
// #define SHM_STAT_ANY    15
numeric_enum! {
    #[repr(u32)]
    #[derive(Eq, PartialEq, Debug, Copy, Clone)]
    pub enum ShmCtlCmd {
        IpcRmid = 0,
        IpcSet = 1,
        IpcStat = 2,
        ShmLock = 11,
        ShmUnlock = 12,
        ShmStat = 13,
        ShmInfo = 14,
        ShmStatAny = 15,
    }
}

// struct shminfo {
//     unsigned long shmmax; /* Maximum segment size */
//     unsigned long shmmin; /* Minimum segment size;
//                                                always 1 */
//     unsigned long shmmni; /* Maximum number of segments */
//     unsigned long shmseg; /* Maximum number of segments
//                                                that a process can attach;
//                                                unused within kernel */
//     unsigned long shmall; /* Maximum number of pages of
//                                                shared memory, system-wide */
// };
#[repr(C)]
pub struct ShamInfo {
    pub shmmax: i32,
    pub shmmin: i32,
    pub shmmni: i32,
    pub shmseg: i32,
    pub shmall: i32,
}

// struct shm_info {
//     int           used_ids; /* # of currently existing
//                                                  segments */
//     unsigned long shm_tot;  /* Total number of shared
//                                                  memory pages */
//     unsigned long shm_rss;  /* # of resident shared
//                                                  memory pages */
//     unsigned long shm_swp;  /* # of swapped shared
//                                                  memory pages */
//     unsigned long swap_attempts;
//     /* Unused since Linux 2.4 */
//     unsigned long swap_successes;
//     /* Unused since Linux 2.4 */
// };

#[repr(C)]
pub struct ShmInfo2 {
    pub used_ids: i32,
    pub shm_tot: u32,
    pub shm_rss: u32,
    pub shm_swp: u32,
    pub swap_attempts: u32,
    pub swap_successes: u32,
}

// struct ShmidDs {
//     struct IpcPerm shm_perm;    /* Ownership and permissions */
//     size_t          shm_segsz;   /* Size of segment (bytes) */
//     time_t          shm_atime;   /* Last attach time */
//     time_t          shm_dtime;   /* Last detach time */
//     time_t          shm_ctime;   /* Creation time/time of last
//                                                modification via shmctl() */
//     pid_t           shm_cpid;    /* PID of creator */
//     pid_t           shm_lpid;    /* PID of last shmat(2)/shmdt(2) */
//     shmatt_t        shm_nattch;  /* No. of current attaches */
//     ...
// };

pub struct ShmIdDs {
    pub shm_perm: IpcPerm,
    pub shm_segsz: usize,
    pub shm_atime: i64,
    pub shm_dtime: i64,
    pub shm_ctime: i64,
    pub shm_cpid: i32,
    pub shm_lpid: i32,
    pub shm_nattch: i32,
}

#[repr(C)]
pub struct IpcPerm {
    key: i32,
    uid: u32,
    gid: u32,
    cuid: u32,
    cgid: u32,
    mode: u16,
    seq: u16,
}

#[repr(C)]
pub struct ShmidDs {
    pub shm_perm: IpcPerm,
    pub shm_segsz: i32,
    pub shm_atime: i32,
    pub shm_dtime: i32,
    pub shm_ctime: i32,
    pub shm_cpid: i32,
    pub shm_lpid: i32,
    pub shm_nattch: u16,
    pub shm_unused: u16,
    pub shm_unused2: usize,
    pub shm_unused3: usize,
}

pub const IPC_PRIVATE: usize = 0;
