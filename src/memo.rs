use std::io::{ErrorKind, Read, Write};
use std::mem::MaybeUninit;

use crate::protocol::{FrameDescriptor, MsgType, FRAME_DESC_SIZE_BYTES, MAX_MSG_SIZE_BYTES};

const STACK_BUF_SIZE: u32 = 1 << 13;
#[derive(Debug)]
pub struct SleamBuf {
    data: Box<[u8]>,
    read_pos: usize,
    write_pos: usize,
}

impl SleamBuf {
    #[inline]
    pub fn alloc_and_reserve(len: u32, reserve: usize) -> Self {
        Self {
            data: SleamBuf::create_buffer(len + reserve as u32),
            read_pos: reserve,
            write_pos: reserve,
        }
    }

    #[inline]
    pub fn alloc(len: u32) -> Self {
        Self::alloc_and_reserve(len, 0)
    }

    //we may want to do this whithout transmute....
    fn create_buffer(len: u32) -> Box<[u8]> {
        if len > STACK_BUF_SIZE {
            let mut heap_data: Vec<MaybeUninit<u8>> = Vec::with_capacity(len as usize);
            unsafe {
                heap_data.set_len(len as usize);
                std::mem::transmute::<Vec<MaybeUninit<u8>>, Vec<u8>>(heap_data).into_boxed_slice()
            }
        } else {
            let stack_data = [MaybeUninit::uninit(); STACK_BUF_SIZE as usize];
            unsafe {
                Box::new(std::mem::transmute::<
                    [MaybeUninit<u8>; STACK_BUF_SIZE as usize],
                    [u8; STACK_BUF_SIZE as usize],
                >(stack_data))
            }
        }
    }

    #[inline]
    pub fn commit(&mut self, msg_type: MsgType) {
        assert!(self.read_pos == FRAME_DESC_SIZE_BYTES);
        let msg_len = self.len() as u32;
        let frame_desc = FrameDescriptor::build_desc(msg_type, msg_len);
        self.data[0..FRAME_DESC_SIZE_BYTES].copy_from_slice(&frame_desc.to_le_bytes()[0..8]);
        self.read_pos = 0;
    }

    pub fn copy_to(&mut self, out: &mut impl Write) -> std::io::Result<u32> {
        let len = self.len();
        while !self.is_empty() {
            match out.write(self.read_slice()) {
                Ok(n) => self.read_pos += n,
                Err(err) => match err.kind() {
                    ErrorKind::WouldBlock => return Ok((len - self.len()) as u32),
                    ErrorKind::Interrupted => continue,
                    _ => return Err(err),
                },
            }
        }
        assert!(self.is_empty());
        Ok(len as u32)
    }

    pub fn copy_from(&mut self, input: &mut impl Read, len: u32) -> std::io::Result<u32> {
        let mut left = len;
        while left > 0 {
            match input.read(&mut self.write_slice()[..left as usize]) {
                Ok(n) => {
                    assert!(len >= n as u32);
                    self.write_pos += n;
                    left -= n as u32;
                }
                Err(err) => match err.kind() {
                    ErrorKind::WouldBlock => return Ok(len - left),
                    ErrorKind::Interrupted => continue,
                    _ => return Err(err),
                },
            }
        }
        Ok(len)
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.write_pos - self.read_pos
    }

    #[inline(always)]
    pub fn write_available(&self) -> usize {
        self.data.len() - self.write_pos
    }

    #[inline(always)]
    pub fn write_slice(&mut self) -> &mut [u8] {
        &mut self.data.as_mut()[self.write_pos..]
    }

    #[inline(always)]
    pub fn read_slice(&mut self) -> &mut [u8] {
        &mut self.data.as_mut()[self.read_pos..self.write_pos]
    }

    /// Returns true if there are unread bytes in the buffer.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.write_pos == self.read_pos
    }

    #[inline(always)]
    /// Discards all data in the buffer.
    pub fn clear(&mut self) {
        self.reset(0);
    }

    #[inline(always)]
    pub fn skip(&mut self, to_skip: u32) {
        if self.len() >= to_skip as usize {
            self.read_pos += to_skip as usize;
        } else {
            self.read_pos = self.write_pos;
        }
    }
    #[inline(always)]
    pub fn advance(&mut self, advance_by: usize) {
        self.write_pos += advance_by;
    }

    #[inline(always)]
    pub fn reset(&mut self, reserve: usize) {
        self.read_pos = reserve;
        self.write_pos = reserve;
    }
}

impl Default for SleamBuf {
    fn default() -> Self {
        SleamBuf::alloc_and_reserve(MAX_MSG_SIZE_BYTES, FRAME_DESC_SIZE_BYTES)
    }
}

impl Write for SleamBuf {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if self.write_available() < buf.len() {
            return Err(ErrorKind::OutOfMemory.into());
        }
        let from = self.write_pos;
        let to = from + buf.len();
        self.data[from..to].copy_from_slice(buf);
        self.write_pos = to;
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl Read for SleamBuf {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let to_read = std::cmp::min(self.len(), buf.len());
        buf[0..to_read].copy_from_slice(&self.data[self.read_pos..self.read_pos + to_read]);
        self.read_pos += to_read;
        Ok(to_read)
    }
}
