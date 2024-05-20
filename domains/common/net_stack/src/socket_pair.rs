use alloc::{collections::VecDeque, sync::Arc};
use core::cmp;

use basic::{constants::io::PollEvents, sync::Mutex, AlienError, AlienResult};

use crate::SocketFile;

pub struct SocketPair {
    inner: Arc<Mutex<VecDeque<u8>>>,
}

impl SocketPair {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            inner: Arc::new(Mutex::new(VecDeque::new())),
        })
    }
}

impl SocketFile for SocketPair {
    fn write_at(&self, _offset: usize, buffer: &[u8]) -> AlienResult<usize> {
        let mut queue = self.inner.lock();
        if queue.len() > 0x50000 {
            Err(AlienError::EBLOCKING)
        } else {
            let wlen = buffer.len();
            queue.extend(buffer.iter());
            Ok(wlen)
        }
    }

    fn read_at(&self, _offset: usize, buffer: &mut [u8]) -> AlienResult<usize> {
        let mut queue = self.inner.lock();
        let rlen = cmp::min(queue.len(), buffer.len());
        queue
            .drain(..rlen)
            .enumerate()
            .into_iter()
            .for_each(|(i, x)| {
                buffer[i] = x;
            });
        if rlen == 0 {
            Err(AlienError::EBLOCKING)
        } else {
            Ok(rlen)
        }
    }

    fn poll(&self, events: PollEvents) -> AlienResult<PollEvents> {
        let mut res = PollEvents::empty();
        if events.contains(PollEvents::OUT) {
            if self.inner.lock().len() <= 0x50000 {
                res |= PollEvents::OUT;
            }
        }
        if events.contains(PollEvents::IN) {
            if self.inner.lock().len() > 0 {
                res |= PollEvents::IN;
            }
        }
        Ok(res)
    }
}
