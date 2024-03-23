//! 信号的编号
numeric_enum_macro::numeric_enum! {
    #[repr(u8)]
    #[allow(missing_docs)]
    #[derive(Eq, PartialEq, Debug, Copy, Clone)]
    /// 信号编号。
    ///
    /// 从 32 开始的部分为 SIGRT，其中 RT 表示 real time。
    /// 但目前实现时没有通过 ipi 等手段即时处理，而是像其他信号一样等到 trap 再处理
    pub enum SignalNumber {
        ERR = 0,
        SIGHUP = 1,
        SIGINT = 2,
        SIGQUIT = 3,
        SIGILL = 4,
        SIGTRAP = 5,
        SIGABRT = 6,
        SIGBUS = 7,
        SIGFPE = 8,
        SIGKILL = 9,
        SIGUSR1 = 10,
        SIGSEGV = 11,
        SIGUSR2 = 12,
        SIGPIPE = 13,
        SIGALRM = 14,
        SIGTERM = 15,
        SIGSTKFLT = 16,
        SIGCHLD = 17,
        SIGCONT = 18,
        SIGSTOP = 19,
        SIGTSTP = 20,
        SIGTTIN = 21,
        SIGTTOU = 22,
        SIGURG = 23,
        SIGXCPU = 24,
        SIGXFSZ = 25,
        SIGVTALRM = 26,
        SIGPROF = 27,
        SIGWINCH = 28,
        SIGIO = 29,
        SIGPWR = 30,
        SIGSYS = 31,
        SIGRTMIN = 32,
        SIGRT1 = 33,
        SIGRT2 = 34,
        SIGRT3 = 35,
        SIGRT4 = 36,
        SIGRT5 = 37,
        SIGRT6 = 38,
        SIGRT7 = 39,
        SIGRT8 = 40,
        SIGRT9 = 41,
        SIGRT10 = 42,
        SIGRT11 = 43,
        SIGRT12 = 44,
        SIGRT13 = 45,
        SIGRT14 = 46,
        SIGRT15 = 47,
        SIGRT16 = 48,
        SIGRT17 = 49,
        SIGRT18 = 50,
        SIGRT19 = 51,
        SIGRT20 = 52,
        SIGRT21 = 53,
        SIGRT22 = 54,
        SIGRT23 = 55,
        SIGRT24 = 56,
        SIGRT25 = 57,
        SIGRT26 = 58,
        SIGRT27 = 59,
        SIGRT28 = 60,
        SIGRT29 = 61,
        SIGRT30 = 62,
        SIGRT31 = 63,
    }
}

impl From<usize> for SignalNumber {
    fn from(num: usize) -> Self {
        Self::try_from(num as u8).unwrap_or(Self::ERR)
    }
}
