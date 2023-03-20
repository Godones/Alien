#![cfg_attr(not(feature = "std"), no_std)]
#![allow(non_snake_case)]

#[cfg(feature = "std")]
pub mod scan;

#[cfg(test)]
extern crate std;

#[cfg(feature = "alloc")]
extern crate alloc;

use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::sync::Arc;
pub use systable_macro_derive::syscall_func;

pub struct SysCallTable {
    table: BTreeMap<usize, Arc<dyn Handler>>,
}

impl SysCallTable {
    pub const fn new() -> Self {
        Self {
            table: BTreeMap::new(),
        }
    }
    pub fn register(&mut self, id: usize, handler: Arc<dyn Handler>) {
        self.table.insert(id, handler);
    }
    pub fn remove(&mut self, id: usize) -> Option<Arc<dyn Handler>> {
        self.table.remove(&id)
    }
}

pub static mut SYSCALL_TABLE: SysCallTable = SysCallTable::new();

pub trait Handler {
    fn name(&self) -> &str;
    fn do_handler(&self, args: [usize; 6]);
}

pub trait UniFn<Args, Res> {
    fn call(&self, args: Args) -> Res;
}

macro_rules! unifn_tuple {
    ($(($arg:ident,$n:tt)),+) => {
        impl<T,$($arg,)+ Res> UniFn<($($arg,)+),Res> for T
        where
            T: Fn($($arg,)+)->Res
        {
            fn call(&self,args:($($arg,)+))->Res{
                (self)($(args.$n,)+)
            }
        }
    };
}
impl<T, Res> UniFn<(), Res> for T
where
    T: Fn() -> Res,
{
    fn call(&self, _: ()) -> Res {
        (self)()
    }
}
unifn_tuple!((P0, 0));
unifn_tuple!((P0, 0), (P1, 1));
unifn_tuple!((P0, 0), (P1, 1), (P2, 2));
unifn_tuple!((P0, 0), (P1, 1), (P2, 2), (P3, 3));
unifn_tuple!((P0, 0), (P1, 1), (P2, 2), (P3, 3), (P4, 4));
unifn_tuple!((P0, 0), (P1, 1), (P2, 2), (P3, 3), (P4, 4), (P5, 5));
unifn_tuple!(
    (P0, 0),
    (P1, 1),
    (P2, 2),
    (P3, 3),
    (P4, 4),
    (P5, 5),
    (P6, 6)
);

pub struct SysCallHandler<F, Args, Res> {
    func: F,
    _args: core::marker::PhantomData<Args>,
    _res: core::marker::PhantomData<Res>,
}

impl<F, Args, Res> SysCallHandler<F, Args, Res>
where
    F: UniFn<Args, Res>,
{
    pub fn new(func: F) -> Self {
        Self {
            func,
            _args: core::marker::PhantomData,
            _res: core::marker::PhantomData,
        }
    }
    pub fn call(&self, args: Args) -> Res {
        self.func.call(args)
    }
}

pub trait FromArgs: Sized {
    fn from(args: &[usize]) -> Result<Self, String>;
}

impl FromArgs for () {
    fn from(_: &[usize]) -> Result<Self, String> {
        Ok(())
    }
}

macro_rules! mark_basic_type {
    ($ident:ty) => {
        impl FromArgs for $ident {
            fn from(args: &[usize]) -> Result<Self, String> {
                if args.len() >= 1 {
                    let res = args[0] as $ident;
                    Ok(res)
                } else {
                    Err(crate::alloc::format!(
                        "{}:args.len() < 1",
                        stringify!($ident)
                    ))
                }
            }
        }
    };
}
mark_basic_type!(usize);
mark_basic_type!(u64);
mark_basic_type!(u32);
mark_basic_type!(u16);
mark_basic_type!(u8);
mark_basic_type!(isize);
mark_basic_type!(i64);
mark_basic_type!(i32);
mark_basic_type!(i16);
mark_basic_type!(i8);
mark_basic_type!(*mut u8);
mark_basic_type!(*const u8);
mark_basic_type!(*const usize);
mark_basic_type!(*mut usize);
mark_basic_type!(*mut u32);
mark_basic_type!(*const u32);
mark_basic_type!(*mut u64);
mark_basic_type!(*const u64);
mark_basic_type!(*mut i8);
mark_basic_type!(*const i8);
mark_basic_type!(*mut i16);
mark_basic_type!(*const i16);
mark_basic_type!(*mut i32);
mark_basic_type!(*const i32);
mark_basic_type!(*mut i64);
mark_basic_type!(*const i64);
mark_basic_type!(*mut isize);
mark_basic_type!(*const isize);

macro_rules! from_args_tuple {
    ($(($arg:ident,$n:tt)),+) => {
        impl<$($arg,)+> FromArgs for ($($arg,)+)
        where
            $($arg:FromArgs,)+
        {
            fn from(args:&[usize])->Result<Self,String>{
                $(let $arg = $arg::from(&args[$n..])?;)+
                Ok(($($arg,)+))
            }
        }
    };
}

from_args_tuple!((P0, 0));
from_args_tuple!((P0, 0), (P1, 1));
from_args_tuple!((P0, 0), (P1, 1), (P2, 2));
from_args_tuple!((P0, 0), (P1, 1), (P2, 2), (P3, 3));
from_args_tuple!((P0, 0), (P1, 1), (P2, 2), (P3, 3), (P4, 4));
from_args_tuple!((P0, 0), (P1, 1), (P2, 2), (P3, 3), (P4, 4), (P5, 5));

pub struct Service {
    service: Box<dyn Fn(&[usize]) -> isize>,
}

impl Service {
    pub fn from_handler<F, Args, Res>(handler: SysCallHandler<F, Args, Res>) -> Self
    where
        F: UniFn<Args, Res> + 'static,
        Args: FromArgs + 'static,
        Res: Into<isize> + 'static,
    {
        Self {
            service: Box::new(move |args: &[usize]| {
                let args = Args::from(args).unwrap();
                handler.call(args).into()
            }),
        }
    }
    pub fn handle(&self, args: &[usize]) -> isize {
        (self.service)(args)
    }
}

pub struct Table {
    map: BTreeMap<usize, Service>,
}
unsafe impl Send for Table {}
unsafe impl Sync for Table {}

impl Table {
    pub const fn new() -> Self {
        Self {
            map: BTreeMap::new(),
        }
    }
    pub fn register<F, Args, Res>(&mut self, id: usize, func: F)
    where
        F: UniFn<Args, Res> + 'static,
        Args: FromArgs + 'static,
        Res: Into<isize> + 'static,
    {
        let handler = SysCallHandler::new(func);
        self.map.insert(id, Service::from_handler(handler));
    }
    pub fn remove(&mut self, id: usize) -> Option<Service> {
        self.map.remove(&id)
    }
    pub fn do_call(&self, id: usize, args: &[usize]) -> Option<isize> {
        self.map.get(&id).map(|x| x.handle(args))
    }
}

#[macro_export]
macro_rules! register_syscall {
    ($table:ident,$(($id:expr,$func:ident)),+ $(,)?) => {
        $(
            $table.register($id,$func);
        )+
    };
    ($table:ident,)=>{

    }
}

#[cfg(test)]
mod tests {
    use super::Table;
    use std::println;
    use std::vec::Vec;
    fn read(p1: usize, p2: usize) -> isize {
        println!("p1+p2 = {}", p1 + p2);
        0
    }
    fn test(p1: usize, p2: usize, p3: *const u8) -> isize {
        let len = p1 + p2;
        let buf = unsafe { core::slice::from_raw_parts(p3, len) };
        // transfer to usize
        let buf = buf
            .chunks(8)
            .map(|x| {
                let mut buf = [0u8; 8];
                buf.copy_from_slice(x);
                usize::from_le_bytes(buf)
            })
            .collect::<Vec<usize>>();
        println!("read {}, buf = {:?}", len, buf);
        0
    }

    #[test]
    fn table_test() {
        let mut table = Table::new();
        table.register(0, read);
        table.register(1, test);
        table.do_call(0, &[1, 2, 0, 0, 0, 0]);
        let data = [6usize; 8];
        table.do_call(1, &[0, 8 * 8, data.as_ptr() as usize, 0, 0, 0]);
    }
}
