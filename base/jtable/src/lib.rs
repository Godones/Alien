#![no_std]
#![feature(specialization)]
#![cfg_attr(target_arch = "riscv64", feature(riscv_ext_intrinsics))]
#![allow(incomplete_features)]

mod arch;
use core::{fmt::Debug, sync::atomic::AtomicBool};

pub use arch::*;

#[repr(C)]
#[derive(Debug)]
pub struct StaticKey {
    enabled: AtomicBool,
}

impl StaticKey {
    pub const fn default_true() -> Self {
        StaticKey {
            enabled: AtomicBool::new(true),
        }
    }
    pub const fn default_false() -> Self {
        StaticKey {
            enabled: AtomicBool::new(false),
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled.load(core::sync::atomic::Ordering::Relaxed)
    }

    pub fn set_enabled(&self, enabled: bool) {
        self.enabled
            .store(enabled, core::sync::atomic::Ordering::Relaxed)
    }
}

#[derive(Debug)]
pub struct StaticKeyTrue(pub StaticKey);

impl StaticKeyTrue {
    pub const fn new() -> Self {
        StaticKeyTrue(StaticKey::default_true())
    }
}

#[derive(Debug)]
pub struct StaticKeyFalse(pub StaticKey);

impl StaticKeyFalse {
    pub const fn new() -> Self {
        StaticKeyFalse(StaticKey::default_false())
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum StaticKeyType {
    StaticKeyTrue,
    StaticKeyFalse,
    Other,
}

pub trait StaticKeyTypeTrait {
    fn static_key_type(&self) -> StaticKeyType;
}

impl<T> StaticKeyTypeTrait for T {
    #[inline]
    default fn static_key_type(&self) -> StaticKeyType {
        StaticKeyType::Other
    }
}

impl StaticKeyTypeTrait for StaticKeyTrue {
    #[inline]
    fn static_key_type(&self) -> StaticKeyType {
        StaticKeyType::StaticKeyTrue
    }
}

impl StaticKeyTypeTrait for StaticKeyFalse {
    #[inline]
    fn static_key_type(&self) -> StaticKeyType {
        StaticKeyType::StaticKeyFalse
    }
}

#[macro_export]
macro_rules! static_branch_likely {
    ($key:ident) => {{
        if $key.static_key_type() == StaticKeyType::StaticKeyTrue {
            unsafe {
                paste! {
                    [<$key _is_false>]()
                }
            }
        } else if $key.static_key_type() == StaticKeyType::StaticKeyFalse {
            unsafe {
                paste! {
                    [<$key _is_false>]()
                }
            }
        } else {
            panic!("static key is not true or false")
        }
    }};
}

#[macro_export]
macro_rules! static_branch_unlikely {
    ($key:ident) => {{
        if $key.static_key_type() == StaticKeyType::StaticKeyTrue {
            unsafe {
                paste! {
                    ![<$key _is_false>]()
                }
            }
        } else if $key.static_key_type() == StaticKeyType::StaticKeyFalse {
            unsafe {
                paste! {
                    ![<$key _is_false>]()
                }
            }
        } else {
            panic!("static key is not true or false")
        }
    }};
}

#[macro_export]
macro_rules! static_branch_enable {
    ($key:ident) => {{
        static_key_enable(
            &$key.0,
            paste! {
                    [<$key _is_false>]
            } as usize,
        )
    }};
}

#[macro_export]
macro_rules! static_branch_disable {
    ($key:ident) => {{
        static_key_disable(
            &$key.0,
            paste! {
                    [<$key _is_false>]
            } as usize,
        )
    }};
}
