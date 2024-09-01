#![no_std]
#![feature(specialization)]
#![cfg_attr(target_arch = "riscv64", feature(riscv_ext_intrinsics))]
#![allow(incomplete_features)]

mod arch;
use core::{fmt::Debug, sync::atomic::AtomicBool};

pub use arch::*;

#[repr(C)]
#[derive(Debug)]
pub struct StaticKey<const T: bool> {
    enabled: AtomicBool,
}

impl<const T: bool> StaticKey<T> {
    pub const fn new() -> Self {
        Self {
            enabled: AtomicBool::new(false),
        }
    }

    pub const fn is_true(&self) -> bool {
        T
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled.load(core::sync::atomic::Ordering::Relaxed)
    }

    pub fn set_enabled(&self, enabled: bool) {
        self.enabled
            .store(enabled, core::sync::atomic::Ordering::Relaxed)
    }
}

pub type StaticKeyTrue = StaticKey<true>;

pub type StaticKeyFalse = StaticKey<false>;

#[derive(Debug, Eq, PartialEq)]
pub enum StaticKeyType {
    StaticKeyTrue,
    StaticKeyFalse,
    Other,
}

#[macro_export]
macro_rules! static_branch_likely {
    ($key:ident) => {{
        if $key.is_true() {
            unsafe {
                paste! {
                    [<$key _is_false>]()
                }
            }
        } else {
            unsafe {
                paste! {
                    [<$key _is_false>]()
                }
            }
        }
    }};
}

#[macro_export]
macro_rules! static_branch_unlikely {
    ($key:ident) => {{
        if $key.is_true() {
            unsafe {
                paste! {
                    ![<$key _is_false>]()
                }
            }
        } else {
            unsafe {
                paste! {
                    ![<$key _is_false>]()
                }
            }
        }
    }};
}

#[macro_export]
macro_rules! static_branch_enable {
    ($key:ident) => {{
        static_key_enable(
            &$key,
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
            &$key,
            paste! {
                    [<$key _is_false>]
            } as usize,
        )
    }};
}
