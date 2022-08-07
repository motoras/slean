use byteorder::{ByteOrder, LittleEndian};
use mio::net::TcpStream;

use std::io::{Error, ErrorKind, Read};

use crate::memo::SleamBuf;
use crate::protocol::{FrameDescriptor, FRAME_DESC_SIZE_BYTES};
use crate::service::{IoBuf, ReplService};

pub struct Connection<'a, S: ReplService> {
    pub(crate) stream: TcpStream,
    service: &'a S,
    pending_read: u32,
    pending_buff: Option<SleamBuf>,
}

impl<'a, S: ReplService> Connection<'a, S> {
    pub fn new(stream: TcpStream, service: &'a S) -> Self {
        Connection {
            stream,
            service,
            pending_read: 0,
            pending_buff: None,
        }
    }

    #[inline]
    pub fn is_write_pending(&self) -> bool {
        self.pending_read == 0 && self.pending_buff.is_some()
    }

    #[inline]
    pub fn on_write(&mut self) -> Result<u32, Error> {
        if self.pending_read == 0 {
            match &mut self.pending_buff {
                Some(crt_write_buff) => match crt_write_buff.copy_to(&mut self.stream) {
                    Ok(n) => {
                        if crt_write_buff.is_empty() {
                            self.pending_buff = None;
                        }
                        return Ok(n);
                    }
                    Err(e) => return Err(e),
                },
                None => return Ok(0),
            }
        }
        Ok(0)
    }

    #[inline]
    pub fn is_read_pending(&self) -> bool {
        self.pending_read > 0
    }
    pub fn on_read(&mut self, rw_buff: &mut SleamBuf) -> Result<u32, Error> {
        assert!(!self.is_write_pending());
        if self.pending_read > 0 {
            //we must read in the connection's buffer
            assert!(self.pending_buff.is_some());
            match &mut self.pending_buff {
                Some(crt_read_buff) => {
                    match crt_read_buff.copy_from(&mut self.stream, self.pending_read) {
                        Ok(n) => {
                            self.pending_read -= n;
                            if self.pending_read == 0 {
                                assert!(crt_read_buff.write_available() == 0);
                                self.service
                                    .execute(&mut IoBuf::Separate(crt_read_buff, rw_buff));
                                self.pending_buff = None;
                            } else {
                                //we still need to read more
                                return Ok(n);
                            }
                        }
                        Err(e) => return Err(e),
                    }
                }
                None => return Ok(0),
            }
        }

        loop {
            let msg_len = self.read_message_length()?;
            if msg_len == 0 {
                return Ok(0);
            }
            while self.pending_read > 0 {
                match rw_buff.copy_from(&mut self.stream, self.pending_read) {
                    Ok(n) => {
                        self.pending_read -= n;
                        if self.pending_read == 0 {
                            self.service.execute(&mut IoBuf::Same(rw_buff));
                            rw_buff.copy_to(&mut self.stream);
                            let _ = self.send(rw_buff)?;
                            if self.pending_buff.is_some() {
                                return Ok(n); //write was incomplete so will need to try again
                            }
                        } else {
                            //read was incomplete we still need to read more
                            let mut conn_buf = SleamBuf::alloc(msg_len);
                            conn_buf.copy_from(rw_buff, rw_buff.len() as u32);
                            rw_buff.clear();
                            self.pending_buff = Some(conn_buf);
                            return Ok(n);
                        }
                    }
                    Err(e) => return Err(e),
                }
            }
        }
    }

    fn send(&mut self, rw_buff: &mut SleamBuf) -> Result<u32, Error> {
        match rw_buff.copy_to(&mut self.stream) {
            Ok(n) => {
                let left = rw_buff.len() as u32;
                if left > 0 {
                    let mut conn_buf = SleamBuf::alloc(left);
                    conn_buf.copy_from(rw_buff, rw_buff.len() as u32);
                    self.pending_buff = Some(conn_buf);
                }
                rw_buff.clear();
                return Ok(n);
            }
            Err(e) => return Err(e),
        }
    }

    #[inline]
    fn read_message_length(&mut self) -> std::io::Result<u32> {
        self.pending_read = 0;
        //frame delim 4 bytes metadata 4 bytes frame info
        let mut buf = [0u8; FRAME_DESC_SIZE_BYTES];
        //we want to read the entire descriptor
        let bytes_read = self.stream.peek(&mut buf)?;
        if bytes_read < FRAME_DESC_SIZE_BYTES {
            return Ok(0); //will read later
        }
        //skip the peeked bytes; this shall not crash
        std::io::copy(
            &mut self.stream.by_ref().take(FRAME_DESC_SIZE_BYTES as u64),
            &mut std::io::sink(),
        )?;
        let descr = LittleEndian::read_u64(buf.as_ref());
        let frame_desc: FrameDescriptor =
            descr.try_into().map_err(|_err| ErrorKind::InvalidData)?;
        if !frame_desc.is_req() {
            return Err(ErrorKind::InvalidData.into());
        }
        self.pending_read = frame_desc.len();
        Ok(self.pending_read)
    }
}
