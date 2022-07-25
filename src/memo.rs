use std::io::{Error, ErrorKind, Write};
use std::mem::MaybeUninit;

use bytes::BufMut;
use mio::net::TcpStream;

pub trait MemoryPool {
    //todo investigate if I can use a lifetime parameter instead of boxing
    fn take(&mut self, len: u32) -> Option<Box<&[u8]>>;
    fn put(&mut self, buff: Box<&[u8]>);
}

#[derive(Debug)]
pub struct LimitedMemoryPool {
    max_size: u32,
    used: u32,
}

impl LimitedMemoryPool {
    pub fn new(max_size: u32) -> Self {
        LimitedMemoryPool {
            max_size,
            used: 0u32,
        }
    }
}

impl MemoryPool for LimitedMemoryPool {
    fn take(&mut self, len: u32) -> Option<Box<&[u8]>> {
        if len + self.used <= self.max_size {
            self.used += len;
            return Some(Box::new(Vec::with_capacity(len as usize).as_ref()));
        }
        None
    }

    fn put(&mut self, buff: Box<&[u8]>) {
        self.used -= buff.len() as u32;
    }
}

pub trait TcpWriter: Write {
    fn send(&mut self, out: &mut TcpStream) -> std::io::Result<()>;
}

struct TcpWriteBuff<'buff> {
    buff: &'buff [u8],
    pos: usize,
}

impl<'buff> TcpWriteBuff<'buff> {
    pub fn new(len: u32) -> Self {
        Self {
            buff: &Vec::with_capacity(std::mem::size_of::<u32>() + len as usize),
            pos: std::mem::size_of::<u32>(),
        }
    }
}

impl<'buff> TcpWriter for TcpWriteBuff<'buff> {
    #[inline]
    fn send(&mut self, out: &mut TcpStream) -> std::io::Result<()> {
        assert!(self.pos > std::mem::size_of::<u32>());
        let buf_len = self.pos;
        self.buff[0..4].copy_from_slice(&(self.pos - std::mem::size_of::<u32>()).to_le_bytes());
        self.pos = std::mem::size_of::<u32>(); //either we succedd or not we will RESET the buffer
        out.write_all(&self.buff[0..self.pos])
    }
}

impl<'buff> Write for TcpWriteBuff<'buff> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if self.pos + buf.len() >= self.buff.len() {
            return Err(ErrorKind::OutOfMemory.into());
        }
        let from = self.pos;
        let to = from + buf.len();
        self.buff[from..to].copy_from_slice(buf);
        self.pos = to;
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.pos = 0;
        Ok(())
    }
}
