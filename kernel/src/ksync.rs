use crate::arch::{hart_id, interrupt_disable, interrupt_enable, is_interrupt_enable};
use crate::config::CPU_NUM;
use core::cell::{RefCell, RefMut};
use kernel_sync::{ticket::TicketMutexGuard, LockAction};

pub type SpinMutex<T> = kernel_sync::spin::SpinMutex<T, KernelLockAction>;
pub type TicketMutex<T> = kernel_sync::ticket::TicketMutex<T, KernelLockAction>;
pub type RwLock<T> = kernel_sync::RwLock<T>;
pub type Mutex<T> = TicketMutex<T>;
pub type MutexGuard<'a, T> = TicketMutexGuard<'a, T, KernelLockAction>;

#[derive(Debug, Default, Clone, Copy)]
#[repr(align(64))]
pub struct Cpu {
    pub noff: i32,              // Depth of push_off() nesting.
    pub interrupt_enable: bool, // Were interrupts enabled before push_off()?
}

impl Cpu {
    const fn new() -> Self {
        Self {
            noff: 0,
            interrupt_enable: false,
        }
    }
}

pub struct SafeRefCell<T>(RefCell<T>);

/// # Safety: Only the corresponding cpu will access it.
unsafe impl<Cpu> Sync for SafeRefCell<Cpu> {}

impl<T> SafeRefCell<T> {
    const fn new(t: T) -> Self {
        Self(RefCell::new(t))
    }
}

#[allow(clippy::declare_interior_mutable_const)]
const DEFAULT_CPU: SafeRefCell<Cpu> = SafeRefCell::new(Cpu::new());

static CPUS: [SafeRefCell<Cpu>; CPU_NUM] = [DEFAULT_CPU; CPU_NUM];

pub fn mycpu() -> RefMut<'static, Cpu> {
    CPUS[hart_id()].0.borrow_mut()
}

// push_off/pop_off are like intr_off()/intr_on() except that they are matched:
// it takes two pop_off()s to undo two push_off()s.  Also, if interrupts
// are initially off, then push_off, pop_off leaves them off.
pub(crate) fn push_off() {
    let old = is_interrupt_enable();
    interrupt_disable();
    let mut cpu = mycpu();
    if cpu.noff == 0 {
        cpu.interrupt_enable = old;
    }
    cpu.noff += 1;
}

pub(crate) fn pop_off() {
    let mut cpu = mycpu();
    if is_interrupt_enable() || cpu.noff < 1 {
        panic!("pop_off");
    }
    cpu.noff -= 1;
    let should_enable = cpu.noff == 0 && cpu.interrupt_enable;
    drop(cpu);
    // NOTICE: intr_on() may lead to an immediate inerrupt, so we *MUST* drop(cpu) in advance.
    if should_enable {
        interrupt_enable();
    }
}

pub struct KernelLockAction;
impl LockAction for KernelLockAction {
    fn before_lock() {
        push_off();
    }
    fn after_lock() {
        pop_off();
    }
}
