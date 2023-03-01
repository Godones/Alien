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
            syscall($id, [0, 0, 0])
        }
    };
    ($name:ident,$id:expr,$t:ty) => {
        pub fn $name(arg: $t) -> isize {
            syscall($id, [arg as usize, 0, 0])
        }
    };
    ($name:ident,$id:expr,$t1:ty,$t2:ty) => {
        pub fn $name(arg1: $t1, arg2: $t2) -> isize {
            syscall($id, [arg1 as usize, arg2 as usize, 0])
        }
    };
    ($name:ident,$id:expr,$t1:ty,$t2:ty,$t3:ty) => {
        pub fn $name(arg1: $t1, arg2: $t2, arg3: $t3) -> isize {
            syscall($id, [arg1 as usize, arg2 as usize, arg3 as usize])
        }
    };
}
