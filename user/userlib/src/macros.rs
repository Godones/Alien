#[macro_export]
macro_rules! syscall_id {
    ($name:ident,$val:expr) => {
        const $name: usize = $val;
    };
}
#[macro_export]
macro_rules! syscall {
    ($name:ident,$id:expr) => {
        pub fn $name() -> isize {
            syscall($id, [0, 0, 0, 0, 0, 0])
        }
    };
    ($name:ident,$id:expr,$t:ty) => {
        pub fn $name(arg: $t) -> isize {
            syscall($id, [arg as usize, 0, 0, 0, 0, 0])
        }
    };
    ($name:ident,$id:expr,$t1:ty,$t2:ty) => {
        pub fn $name(arg1: $t1, arg2: $t2) -> isize {
            syscall($id, [arg1 as usize, arg2 as usize, 0, 0, 0, 0])
        }
    };
    ($name:ident,$id:expr,$t1:ty,$t2:ty,$t3:ty) => {
        pub fn $name(arg1: $t1, arg2: $t2, arg3: $t3) -> isize {
            syscall($id, [arg1 as usize, arg2 as usize, arg3 as usize, 0, 0, 0])
        }
    };
    ($name:ident,$id:expr,$t1:ty,$t2:ty,$t3:ty,$t4:ty) => {
        pub fn $name(arg1: $t1, arg2: $t2, arg3: $t3, arg4: $t4) -> isize {
            syscall(
                $id,
                [
                    arg1 as usize,
                    arg2 as usize,
                    arg3 as usize,
                    arg4 as usize,
                    0,
                    0,
                ],
            )
        }
    };
    ($name:ident,$id:expr,$t1:ty,$t2:ty,$t3:ty,$t4:ty,$t5:ty) => {
        pub fn $name(arg1: $t1, arg2: $t2, arg3: $t3, arg4: $t4, arg5: $t5) -> isize {
            syscall(
                $id,
                [
                    arg1 as usize,
                    arg2 as usize,
                    arg3 as usize,
                    arg4 as usize,
                    arg5 as usize,
                    0,
                ],
            )
        }
    };
    ($name:ident,$id:expr,$t1:ty,$t2:ty,$t3:ty,$t4:ty,$t5:ty,$t6:ty) => {
        pub fn $name(arg1: $t1, arg2: $t2, arg3: $t3, arg4: $t4, arg5: $t5, arg6: $t6) -> isize {
            syscall(
                $id,
                [
                    arg1 as usize,
                    arg2 as usize,
                    arg3 as usize,
                    arg4 as usize,
                    arg5 as usize,
                    arg6 as usize,
                ],
            )
        }
    };
}
