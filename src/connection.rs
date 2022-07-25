use byteorder::{ByteOrder, NetworkEndian};
use log::*;
use mio::net::TcpStream;
use mio::Token;

use std::io::{Error, ErrorKind, Read};

use crate::memo::MemoryPool;

pub struct Connection<'a> {
    token: Token,
    stream: TcpStream,
    pending_read: u32,
    buffer: Option<Box<&'a [u8]>>,
    buffer_pos: usize,
}

impl<'a> Connection<'a> {
    pub fn new(token: Token, stream: TcpStream) -> Self {
        Connection {
            token,
            stream,
            pending_read: 0,
            buffer: None,
            buffer_pos: 0,
        }
    }
    pub fn on_read<T: MemoryPool>(&mut self, mem_pool: &'a mut T) -> Result<u32, Error> {
        let mut bytes_read = 0;
        if self.pending_read > 0 {
            let left_to_read = self.pending_read;
            match self.read_msg() {
                Ok(_) => {
                    bytes_read += left_to_read;
                    //we got message
                    //maybe decode first??
                    //on_message(&self.buffer, &self.stream);
                    self.pending_read = 0;
                    self.buffer_pos = 0;
                    mem_pool.put(self.buffer.unwrap());
                    self.buffer = None;
                }
                Err(err) => match err.kind() {
                    ErrorKind::WouldBlock => return Ok(left_to_read - self.pending_read),
                    _ => return Err(err),
                },
            }
        }

        loop {
            let msg_len = self.read_message_length()?;
            if msg_len == 0 {
                return Ok(0);
            }
            let left_to_read = self.pending_read;
            self.buffer = mem_pool.take(msg_len);
            if self.buffer.is_none() {
                //discard message
                //tell the client you are temporarely unavailable due to memory shortage, and ask him to try later
                return Ok(0);
            }
            match self.read_msg() {
                Ok(_) => {
                    bytes_read += left_to_read;
                    //we got message
                    //maybe decode first??
                    //on_message(&self.buffer, &self.stream);
                    self.pending_read = 0;
                    self.buffer_pos = 0;
                    mem_pool.put(self.buffer.unwrap());
                    self.buffer = None;
                }
                Err(err) => match err.kind() {
                    ErrorKind::WouldBlock => return Ok(left_to_read - self.pending_read),
                    _ => return Err(err),
                },
            }
        }
    }

    fn read_msg(&mut self) -> Result<(), Error> {
        assert!(self.buffer.is_some());
        let buffer = self.buffer.unwrap();
        while self.pending_read > 0 {
            match self.stream.read(&mut buffer[self.buffer_pos as usize..]) {
                Ok(n) => {
                    assert!(self.pending_read >= n as u32);
                    self.pending_read -= n as u32;
                    self.buffer_pos += n;
                }

                Err(err) => match err.kind() {
                    ErrorKind::Interrupted => {
                        continue;
                    }
                    _ => return Err(err),
                },
            }
        }
        Ok(())
    }
    #[inline]
    fn read_message_length(&mut self) -> std::io::Result<u32> {
        //maybe we should do a peek first?
        let mut buf = [0u8; 4];
        let bytes = match self.stream.read(&mut buf) {
            Ok(n) => n,
            Err(e) => {
                if e.kind() == ErrorKind::WouldBlock {
                    return Ok(0);
                } else {
                    return Err(e);
                }
            }
        };
        //I guess we can use peek to avoid this
        if bytes < 4 && bytes > 0 {
            warn!("Found message length of {} bytes", bytes);
            return Err(Error::new(ErrorKind::InvalidData, "Invalid message length"));
        }

        self.pending_read = NetworkEndian::read_u32(buf.as_ref());
        Ok(self.pending_read)
    }
}
