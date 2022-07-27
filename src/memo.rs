use std::io::{Error, ErrorKind, Write};
use std::mem::MaybeUninit;

use bytes::BufMut;
use log::info;
use mio::net::TcpStream;

// pub trait MemoryPool {
//     //todo investigate if I can use a lifetime parameter instead of boxing
//     fn take(&mut self, len: u32) -> Option<Box<&[u8]>>;
//     fn put(&mut self, buff: Box<&[u8]>);
// }

// #[derive(Debug, Clone)]
// pub struct LimitedMemoryPool {
//     max_size: u32,
//     used: u32,
// }

// impl LimitedMemoryPool {
//     pub fn new(max_size: u32) -> Self {
//         LimitedMemoryPool {
//             max_size,
//             used: 0u32,
//         }
//     }
// }

// impl MemoryPool for LimitedMemoryPool {
//     fn take(&mut self, len: u32) -> Option<Box<&[u8]>> {
//         if len + self.used <= self.max_size {
//             self.used += len;
//             return Some(Box::new(Vec::with_capacity(len as usize).as_ref()));
//         }
//         None
//     }

//     fn put(&mut self, buff: Box<&[u8]>) {
//         self.used -= buff.len() as u32;
//     }
// }

pub struct TcpWriteBuff {
    buff: Vec<u8>,
    pos: usize,
}
impl Default for TcpWriteBuff {
    fn default() -> Self {
        TcpWriteBuff::new(2 * 1000000)
    }
}

impl TcpWriteBuff {
    pub fn new(len: u32) -> Self {
        let mut buff = Vec::with_capacity(std::mem::size_of::<u32>() + len as usize);
        buff.resize(buff.capacity(), 0);
        Self {
            buff,
            pos: std::mem::size_of::<u32>(),
        }
    }

    // #[inline]
    // pub fn send(&mut self, out: &mut TcpStream) -> std::io::Result<()> {
    //     assert!(self.pos > std::mem::size_of::<u32>());
    //     let buf_len = self.pos;
    //     self.buff[0..4].copy_from_slice(&(self.pos - std::mem::size_of::<u32>()).to_le_bytes());
    //     self.pos = std::mem::size_of::<u32>();
    //     out.write_all(&self.buff[0..self.pos])
    // }

    pub fn send(&mut self, out: &mut impl Write) -> std::io::Result<()> {
        //info!("Writing on the socket {} bytes", self.pos);
        assert!(self.pos > std::mem::size_of::<u32>());
        self.buff[0..4]
            .copy_from_slice(&(self.pos - std::mem::size_of::<u32>()).to_le_bytes()[0..4]);
        let res = out.write_all(&self.buff[..self.pos]);
        self.pos = std::mem::size_of::<u32>();
        res
    }
}

impl Write for TcpWriteBuff {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        // info!(
        //     "Writing {} {} {} ",
        //     self.pos,
        //     buf.len(),
        //     self.buff.capacity()
        // );
        if self.pos + buf.len() >= self.buff.capacity() {
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
