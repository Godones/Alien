use core::{
    fmt::{Debug, Formatter},
    ops::{Index, IndexMut},
};

use gimli::Register;

#[macro_export]
macro_rules! registers {
    ($struct_name:ident, { $($name:ident = ($val:expr, $disp:expr)),+ $(,)? }) => {
        #[allow(missing_docs)]
        impl $struct_name {
            $(
                pub const $name: Register = Register($val);
            )+
        }

        impl $struct_name {
            /// The name of a register, or `None` if the register number is unknown.
            #[allow(dead_code)]
            pub fn register_name(register: Register) -> Option<&'static str> {
                match register {
                    $(
                        Self::$name => Some($disp),
                    )+
                    _ => return None,
                }
            }
        }
    };
}

#[derive(Clone)]
pub struct Registers {
    registers: [Option<u64>; 32],
}

impl Default for Registers {
    fn default() -> Self {
        Registers {
            registers: [None; 32],
        }
    }
}

impl Debug for Registers {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        for (i, reg) in self.registers.iter().enumerate() {
            match *reg {
                None => {} // write!(fmt, "[{}]: None, ", i)?,
                Some(r) => write!(f, "[{}]: {:#X}, \n", i, r)?,
            }
        }
        Ok(())
    }
}

impl Index<gimli::Register> for Registers {
    type Output = Option<u64>;
    fn index(&self, index: Register) -> &Self::Output {
        &self.registers[index.0 as usize]
    }
}

impl IndexMut<gimli::Register> for Registers {
    fn index_mut(&mut self, index: Register) -> &mut Self::Output {
        &mut self.registers[index.0 as usize]
    }
}
