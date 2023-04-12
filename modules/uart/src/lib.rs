#![cfg_attr(not(test), no_std)]
#![allow(unused)]
extern crate alloc;
use alloc::vec;
use alloc::vec::Vec;
use core::fmt::Error;
use core::fmt::Write;
use spin::Mutex;

const BUFFER_SIZE: usize = 128;

#[derive(Clone)]
pub struct UartRaw {
    base: usize,
}

pub struct Uart<T: Send> {
    inner: Mutex<UartInner<T>>,
}
struct UartInner<T: Send> {
    device: UartRaw,
    buffer: [u8; BUFFER_SIZE],
    read_pos: usize,
    write_pos: usize,
    read_queue: Vec<T>,
    write_queue: Vec<T>,
    write_buffer: [u8; BUFFER_SIZE],
    write_head: usize,
    write_tail: usize,
}

#[derive(Debug)]
pub enum UartInterruptType {
    ReceiveDataAvailable = 0x4,
    TransmitHoldingRegisterEmpty = 0x2,
    ModemStatus = 0x0,
    LineStatus = 0x6,
    CharacterTimeout = 0xc,
}

impl UartRaw {
    pub const fn new(base: usize) -> Self {
        Self { base }
    }
    pub fn init(&self) {
        let ptr = self.base as *mut u8;
        unsafe {
            //disable interrupts
            ptr.add(1).write_volatile(0);
            // // special mode to set baud rate.
            // ptr.add(3).write_volatile(0x80);
            // // LSB for baud rate of 38.4K
            // ptr.add(0).write_volatile(0x03);
            // // MSB for baud rate of 38.4k
            // ptr.add(1).write_volatile(0x00);
            // // set the world length to 8 bits
            ptr.add(3).write_volatile(3);
            // reset and enable FIFOs.
            ptr.add(2).write_volatile(0x7);
            // enable receive interrupts and transmit interrupts
            ptr.add(1).write_volatile(0x3);
            // ptr.add(1).write_volatile(0x1);
        }
    }
    pub fn interrupt_type(&self) -> UartInterruptType {
        let ptr = self.base as *mut u8;
        let value = unsafe { ptr.add(2).read_volatile() };
        match value & 0xf {
            0x4 => UartInterruptType::ReceiveDataAvailable,
            0x2 => UartInterruptType::TransmitHoldingRegisterEmpty,
            0x0 => UartInterruptType::ModemStatus,
            0x6 => UartInterruptType::LineStatus,
            0xc => UartInterruptType::CharacterTimeout,
            _ => panic!("Unknown interrupt type"),
        }
    }
    pub fn disable_interrupt(&self, interrupt_type: UartInterruptType) {
        let ptr = self.base as *mut u8;
        let value = unsafe { ptr.add(1).read_volatile() };
        let value = match interrupt_type {
            UartInterruptType::ReceiveDataAvailable => value & !0x1,
            UartInterruptType::TransmitHoldingRegisterEmpty => value & !0x2,
            UartInterruptType::ModemStatus => value & !0x8,
            UartInterruptType::LineStatus => value & !0x4,
            _ => {
                panic!("Unsupported interrupt type");
            }
        };
        unsafe {
            ptr.add(1).write_volatile(value);
        }
    }
    pub fn enable_interrupt(&self, interrupt: UartInterruptType) {
        let ptr = self.base as *mut u8;
        let value = unsafe { ptr.add(1).read_volatile() };
        let value = match interrupt {
            UartInterruptType::ReceiveDataAvailable => value | 0x1,
            UartInterruptType::TransmitHoldingRegisterEmpty => value | 0x2,
            UartInterruptType::ModemStatus => value | 0x8,
            UartInterruptType::LineStatus => value | 0x4,
            _ => {
                panic!("Unsupported interrupt type");
            }
        };
        unsafe {
            ptr.add(1).write_volatile(value);
        }
    }
}

impl Write for UartRaw {
    fn write_str(&mut self, out: &str) -> Result<(), Error> {
        let mut buffer = [0u8; 4];
        for c in out.chars() {
            for code_point in c.encode_utf8(&mut buffer).as_bytes().iter() {
                loop {
                    let ch = self.put(*code_point);
                    if ch.is_none() {
                        break;
                    }
                }
            }
        }
        Ok(())
    }
}

impl UartRaw {
    /// if the transmit holding register is empty, write the byte to it.
    /// otherwise, return the byte.
    #[inline]
    pub fn put(&self, out: u8) -> Option<u8> {
        let ptr = self.base as *mut u8;
        unsafe {
            let c = ptr.add(5).read_volatile();
            return if c & (1 << 5) != 0 {
                // if the transmit holding register is empty,
                // write the byte to it.
                ptr.add(0).write_volatile(out);
                None
            } else {
                // otherwise, return the byte.
                Some(out)
            };
        }
    }
    #[inline]
    pub fn get(&self) -> Option<u8> {
        let ptr = self.base as *mut u8;
        unsafe {
            if ptr.add(5).read_volatile() & 1 == 0 {
                // The DR bit is 0, meaning no data
                None
            } else {
                // The DR bit is 1, meaning data!
                Some(ptr.add(0).read_volatile())
            }
        }
    }
}

impl<T: Send> Uart<T> {
    /// After call the new, we need call the init function
    pub const fn new(base: usize) -> Self {
        Self {
            inner: Mutex::new(UartInner {
                device: UartRaw::new(base),
                buffer: [0; BUFFER_SIZE],
                read_pos: 0,
                write_pos: 0,
                read_queue: vec![],
                write_queue: vec![],
                write_buffer: [0; BUFFER_SIZE],
                write_head: 0,
                write_tail: 0,
            }),
        }
    }
    pub fn init(&self) {
        self.inner.lock().device.init();
    }
}

impl<T: Send> Uart<T> {
    /// **Please use the put_byte function instead of this function**
    ///
    /// print the char to the uart, the wait function is used to make current thread sleep,
    /// the schedule function is used to schedule the thread
    pub fn put_ch<F, F1>(&self, c: u8, wait: F, schedule: F1)
    where
        F: Fn(&mut Vec<T>),
        F1: Fn(),
    {
        loop {
            let mut inner = self.inner.lock();
            if inner.write_head == (inner.write_tail + 1) % BUFFER_SIZE {
                let write_queue = &mut inner.write_queue;
                wait(write_queue);
                drop(inner);
                schedule();
            } else {
                let index = inner.write_tail;
                inner.write_buffer[index] = c;
                inner.write_tail = (index + 1) % BUFFER_SIZE;
                // when the buf if full,we need enable the interrupt
                let device = inner.device.clone();
                drop(inner);
                device.enable_interrupt(UartInterruptType::TransmitHoldingRegisterEmpty);
                // when the interrupt is enabled, the interrupt handler will be called immediately,
                // so we need drop the lock before enable the interrupt.
                return;
            }
        }
    }

    /// the batch version of put_ch
    pub fn put_bytes<F, F1>(&self, bytes: &[u8], wait: F, schedule: F1)
    where
        F: Fn(&mut Vec<T>),
        F1: Fn(),
    {
        let mut buf = bytes;
        loop {
            if buf.len() == 0 {
                return;
            }
            let mut inner = self.inner.lock();
            let len = buf.len();
            let mut index = 0;
            if inner.write_head == (inner.write_tail + 1) % BUFFER_SIZE {
                let write_queue = &mut inner.write_queue;
                wait(write_queue);
                drop(inner);
                schedule();
            } else {
                while index < len && inner.write_head != (inner.write_tail + 1) % BUFFER_SIZE {
                    let mut w_index = inner.write_tail;
                    inner.write_buffer[w_index] = buf[index];
                    inner.write_tail = (w_index + 1) % BUFFER_SIZE;
                    index += 1;
                }
                let device = inner.device.clone();
                drop(inner);
                device.enable_interrupt(UartInterruptType::TransmitHoldingRegisterEmpty);
                if index == len {
                    return;
                } else {
                    buf = &buf[index..];
                }
            }
        }
    }

    /// get a char from the read buffer, if the buffer is empty, the wait function will be called,
    /// the schedule function will be called when the wait function return.
    pub fn get_ch<F, F1>(&self, wait: F, schedule: F1) -> Option<u8>
    where
        F: Fn(&mut Vec<T>),
        F1: Fn(),
    {
        loop {
            let mut inner = self.inner.lock();
            if inner.read_pos == inner.write_pos {
                let read_queue = &mut inner.read_queue;
                wait(read_queue);
                drop(inner);
                schedule();
            } else {
                let c = inner.buffer[inner.read_pos];
                inner.read_pos = (inner.read_pos + 1) % BUFFER_SIZE;
                return Some(c);
            }
        }
    }
}

impl<T: Send> Uart<T> {
    pub fn hand_irq<F>(&self, wakeup: F)
    where
        F: Fn(Vec<T>),
    {
        let mut inner = self.inner.lock();
        // check the type of interrupt
        let interrupt_type = inner.device.interrupt_type();
        match interrupt_type {
            UartInterruptType::ReceiveDataAvailable => {
                // read the data from the device
                let mut count = 0;
                while let Some(c) = inner.device.get() {
                    let index = inner.write_pos;
                    inner.buffer[index] = c;
                    inner.write_pos = (index + 1) % BUFFER_SIZE;
                    count += 1;
                }
                let read_queue = inner.read_queue.drain(..count).collect::<Vec<T>>();
                wakeup(read_queue);
            }
            UartInterruptType::TransmitHoldingRegisterEmpty => {
                // if the write buffer is empty, close the interrupt
                if inner.write_head == inner.write_tail {
                    inner
                        .device
                        .disable_interrupt(UartInterruptType::TransmitHoldingRegisterEmpty);
                    return;
                }
                // write the data to the device
                while inner.write_head != inner.write_tail {
                    let c = inner.write_buffer[inner.write_head];
                    if inner.device.put(c).is_some() {
                        // the char is not written to the device
                        break;
                    }
                    inner.write_head = (inner.write_head + 1) % BUFFER_SIZE;
                }
                let write_queue = inner.write_queue.drain(..).collect::<Vec<T>>();
                wakeup(write_queue);
            }
            _ => {}
        }
    }
}
