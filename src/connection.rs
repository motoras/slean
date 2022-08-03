use byteorder::{ByteOrder, LittleEndian, NetworkEndian};
use log::*;
use mio::net::TcpStream;
use mio::Token;

use std::io::{Error, ErrorKind, Read};

use crate::memo::TcpWriteBuff;
use crate::service::ReplService;

pub struct Connection<'a, S: ReplService> {
    pub(crate) stream: TcpStream,
    service: &'a S,
    pending_read: u32,
    buffer: [u8; 512],
    buffer_pos: usize,
}

impl<'a, S: ReplService> Connection<'a, S> {
    pub fn new(stream: TcpStream, service: &'a S) -> Self {
        Connection {
            stream,
            service,
            pending_read: 0,
            buffer: [0; 512],
            buffer_pos: 0,
        }
    }
    pub fn on_read(&mut self, write_buff: &mut TcpWriteBuff) -> Result<u32, Error> {
        //info!("Reading message");
        let mut bytes_read = 0;
        if self.pending_read > 0 {
            let left_to_read = self.pending_read;
            match self.read_msg() {
                Ok(_) => {
                    bytes_read += left_to_read;
                    self.service
                        .execute(&mut &self.buffer[..], write_buff)
                        .unwrap();
                    write_buff.send(&mut self.stream).unwrap();
                    //self.buffer = &mut [0; 0];
                    self.pending_read = 0;
                    self.buffer_pos = 0;
                }
                Err(err) => match err.kind() {
                    ErrorKind::WouldBlock => return Ok(left_to_read - self.pending_read),
                    _ => return Err(err),
                },
            }
        }
        // self.read_message_length()?;
        // Ok(4)
        loop {
            let msg_len = self.read_message_length()?;
            if msg_len == 0 {
                return Ok(0);
            }
            let left_to_read = self.pending_read;
            //info!("Message len is {}", left_to_read);
            //self.buffer = &mut [0; 128];
            //self.buffer.resize(msg_len as usize, 0);
            // if self.buffer.is_none() {
            //     //discard message
            //     //tell the client you are temporarely unavailable due to memory shortage, and ask him to try later
            //     return Ok(0);
            // }
            while self.pending_read > 0 {
                match self.read_msg() {
                    Ok(_) => {
                        bytes_read += left_to_read;
                    }
                    Err(err) => match err.kind() {
                        ErrorKind::WouldBlock => return Ok(left_to_read - self.pending_read),
                        _ => return Err(err),
                    },
                }
            }
            //we got message
            //info!("Message Read");
            self.service
                .execute(&mut &self.buffer[..], write_buff)
                .unwrap();
            //info!("Sending reply....");
            write_buff.send(&mut self.stream).unwrap();
            //info!("Reply send ....");
            //self.buffer = &mut [0; 0];
            self.pending_read = 0;
            self.buffer_pos = 0;
        }
    }

    fn read_msg(&mut self) -> Result<(), Error> {
        assert!(!self.buffer.is_empty());
        while self.pending_read > 0 {
            match self.stream.read(
                &mut self.buffer
                    [self.buffer_pos as usize..self.buffer_pos + self.pending_read as usize],
            ) {
                Ok(n) => {
                    assert!(self.pending_read >= n as u32);
                    self.pending_read -= n as u32;
                    self.buffer_pos += n;
                }

                Err(err) => match err.kind() {
                    ErrorKind::Interrupted => {
                        continue;
                    }
                    _ => return Err(err), //Would Block is handle at a higher level
                },
            }
        }
        Ok(())
    }
    #[inline]
    fn read_message_length(&mut self) -> std::io::Result<u32> {
        //maybe we should do a peek first?
        let mut buf = [0u8; 8]; //frame delim 4 bytes metadata 4 bytes frame info
        if self.stream.peek(&mut buf).unwrap() != 8 {
            //wait for more
        }
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

        self.pending_read = LittleEndian::read_u32(buf.as_ref());
        Ok(self.pending_read)
    }
}
