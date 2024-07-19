#[macro_export]
macro_rules! k_static_branch_enable {
    ($ident:ident) => {
        static_branch_enable!($ident);
        let current_cpu = arch::hart_id();
        let mut cpu_mask = usize::MAX & ((1 << config::CPU_NUM) - 1);
        // ignore current cpu_num
        cpu_mask &= !(1 << current_cpu);
        #[cfg(vf2)]
        {
            // ignore hart 0
            cpu_mask &= !(1 << 0);
        }
        // println_color!(31, "enable, remote_fence_i cpu_mask: {:#b}", cpu_mask);
        let res = platform::remote_fence_i(cpu_mask, 0);
        assert_eq!(res.error, 0);
    };
}

#[macro_export]
macro_rules! k_static_branch_disable {
    ($ident:ident) => {
        static_branch_disable!($ident);
        let current_cpu = arch::hart_id();
        let mut cpu_mask = usize::MAX & ((1 << config::CPU_NUM) - 1);
        // ignore current cpu_num
        cpu_mask &= !(1 << current_cpu);
        #[cfg(vf2)]
        {
            // ignore hart 0
            cpu_mask &= !(1 << 0);
        }
        // println_color!(31, "diable, remote_fence_i cpu_mask: {:#b}", cpu_mask);
        let res = platform::remote_fence_i(cpu_mask, 0);
        assert_eq!(res.error, 0);
    };
}
