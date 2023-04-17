use alloc::collections::VecDeque;
use alloc::sync::Arc;
use bitflags::bitflags;
use crate::driver::DeviceBase;
use crate::driver::uart::CharDevice;
use crate::task::{current_process, Process, PROCESS_MANAGER, ProcessState};
use crate::task::schedule::schedule;
use volatile::{ReadOnly, Volatile, WriteOnly};
use crate::sync::{IntrLock, IntrLockGuard};
bitflags! {
    /// InterruptEnableRegiste
    #[derive(Copy, Clone)]
    pub struct IER: u8 {
        const RX_AVAILABLE = 1 << 0;
        const TX_EMPTY = 1 << 1;
    }

    /// LineStatusRegister
   #[derive(Copy, Clone)]
    pub struct LSR: u8 {
        const DATA_AVAILABLE = 1 << 0;
        const THR_EMPTY = 1 << 5;
    }

    /// Model Control Register
   #[derive(Copy, Clone)]
    pub struct MCR: u8 {
        const DATA_TERMINAL_READY = 1 << 0;
        const REQUEST_TO_SEND = 1 << 1;
        const AUX_OUTPUT1 = 1 << 2;
        const AUX_OUTPUT2 = 1 << 3;
    }
}

#[repr(C)]
#[allow(dead_code)]
struct ReadWithoutDLAB {
    /// receiver buffer register
    pub rbr: ReadOnly<u8>,
    /// interrupt enable register
    pub ier: Volatile<IER>,
    /// interrupt identification register
    pub iir: ReadOnly<u8>,
    /// line control register
    pub lcr: Volatile<u8>,
    /// model control register
    pub mcr: Volatile<MCR>,
    /// line status register
    pub lsr: ReadOnly<LSR>,
    /// ignore MSR
    _padding1: ReadOnly<u8>,
    /// ignore SCR
    _padding2: ReadOnly<u8>,
}

#[repr(C)]
#[allow(dead_code)]
struct WriteWithoutDLAB {
    /// transmitter holding register
    pub thr: WriteOnly<u8>,
    /// interrupt enable register
    pub ier: Volatile<IER>,
    /// ignore FCR
    _padding0: ReadOnly<u8>,
    /// line control register
    pub lcr: Volatile<u8>,
    /// modem control register
    pub mcr: Volatile<MCR>,
    /// line status register
    pub lsr: ReadOnly<LSR>,
    /// ignore other registers
    _padding1: ReadOnly<u16>,
}

pub struct NS16550aRaw {
    base_addr: usize,
}

impl NS16550aRaw {
    fn read_end(&mut self) -> &mut ReadWithoutDLAB {
        unsafe { &mut *(self.base_addr as *mut ReadWithoutDLAB) }
    }

    fn write_end(&mut self) -> &mut WriteWithoutDLAB {
        unsafe { &mut *(self.base_addr as *mut WriteWithoutDLAB) }
    }

    pub fn new(base_addr: usize) -> Self {
        Self { base_addr }
    }

    pub fn init(&mut self) {
        let read_end = self.read_end();
        let mut mcr = MCR::empty();
        mcr |= MCR::DATA_TERMINAL_READY;
        mcr |= MCR::REQUEST_TO_SEND;
        mcr |= MCR::AUX_OUTPUT2;
        read_end.mcr.write(mcr);
        let ier = IER::RX_AVAILABLE;
        read_end.ier.write(ier);
    }

    pub fn read(&mut self) -> Option<u8> {
        let read_end = self.read_end();
        let lsr = read_end.lsr.read();
        if lsr.contains(LSR::DATA_AVAILABLE) {
            Some(read_end.rbr.read())
        } else {
            None
        }
    }

    pub fn write(&mut self, ch: u8) {
        let write_end = self.write_end();
        loop {
            if write_end.lsr.read().contains(LSR::THR_EMPTY) {
                write_end.thr.write(ch);
                break;
            }
        }
    }
}



pub struct Uart1 {
    inner:IntrLock<UartInner>,
}

pub struct UartInner{
    uart_raw:NS16550aRaw,
    rx_buf:VecDeque<u8>,
    wait_queue:VecDeque<Arc<Process>>,
}

impl Uart1 {
    pub fn new(base:usize)->Self{
        Self{
            inner:IntrLock::new(UartInner{
                uart_raw: NS16550aRaw::new(base),
                rx_buf:VecDeque::new(),
                wait_queue: VecDeque::new(),
            }),
        }
    }
    pub fn init(&self){
        self.access_inner().uart_raw.init();
    }
    pub fn access_inner(&self)->IntrLockGuard<UartInner>{
        self.inner.lock()
    }
}

impl CharDevice for Uart1 {
    fn put(&self, c: u8) {
        self.access_inner().uart_raw.write(c)
    }

    fn get(&self) -> Option<u8> {
        // check receive buffer is empty
        loop {
            let mut inner = self.access_inner();
            if inner.rx_buf.is_empty(){
                // schedule();
                let process = current_process().unwrap();
                process.update_state(ProcessState::Waiting);
                inner.wait_queue.push_back(process.clone());
                drop(inner);
                schedule();
            }else {
                let c = inner.rx_buf.pop_front().unwrap();
                return Some(c);
            }
        }
    }

    fn put_bytes(&self, bytes: &[u8]) {
        for &b in bytes {
            self.put(b);
        }
    }
}

impl DeviceBase for Uart1{
    fn hand_irq(&self) {
        let mut inner = self.access_inner();
        loop {
            if let Some(c) = inner.uart_raw.read(){
                inner.rx_buf.push_back(c);
                if !inner.wait_queue.is_empty(){
                    let process = inner.wait_queue.pop_front().unwrap();
                    process.update_state(ProcessState::Ready);
                    let mut guard = PROCESS_MANAGER.lock();
                    guard.push_back(process);
                }
            }else {
                break;
            }
        }
    }
}

