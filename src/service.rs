use std::io::{Error, Read, Write};

use rmp_serde::encode::write;
use rmp_serde::from_read;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::error::RemoteError;
use crate::memo::SleamBuf;

pub enum IoBuf<'iobuff> {
    Same(&'iobuff mut SleamBuf),
    Separate(&'iobuff mut SleamBuf, &'iobuff mut SleamBuf),
}

impl<'iobuff> IoBuf<'iobuff> {
    pub(crate) fn read_buf(&mut self) -> &mut SleamBuf {
        match self {
            IoBuf::Same(read_buff) => read_buff,
            IoBuf::Separate(read_buff, _) => read_buff,
        }
    }

    pub(crate) fn write_buf(&mut self) -> &mut SleamBuf {
        match self {
            IoBuf::Same(write_buff) => write_buff,
            IoBuf::Separate(_, write_buff) => write_buff,
        }
    }
}

pub struct MsgPackCodec {}

impl MsgPackCodec {
    pub fn read<Req>(reader: &mut impl Read) -> Result<Req, rmp_serde::decode::Error>
    where
        Req: DeserializeOwned,
    {
        from_read(reader)
    }

    pub fn write<Repl>(repl: Repl, buff: &mut impl Write) -> Result<(), rmp_serde::encode::Error>
    where
        Repl: Serialize,
    {
        write(buff, &repl)
    }
}

pub trait ReplService {
    fn execute(&self, io_buf: &mut IoBuf);
}

pub struct SimpleReplyService<Req, Repl> {
    pub worker: fn(Req) -> Result<Repl, Error>,
}

impl<Req, Repl> ReplService for SimpleReplyService<Req, Repl>
where
    Req: DeserializeOwned,
    Repl: Serialize,
{
    fn execute(&self, io_buf: &mut IoBuf) {
        let read_buff = io_buf.read_buf();
        match MsgPackCodec::read(read_buff) {
            Ok(req) => {
                let write_buff = io_buf.write_buf();
                //we need to do this if read and write buffer are the same
                write_buff.clear();
                match (self.worker)(req) {
                    Ok(repl) => match MsgPackCodec::write(repl, write_buff) {
                        Ok(_) => {
                            write_buff.commit(crate::protocol::MSG_TYPE::REPL);
                        }
                        Err(err) => {
                            write_buff.clear();
                            let r_err = RemoteError::new(0, err.to_string());
                            //it must be infallable
                            MsgPackCodec::write(r_err, write_buff).unwrap();
                            write_buff.commit(crate::protocol::MSG_TYPE::ERR);
                        }
                    },
                    Err(err) => {
                        let r_err = RemoteError::new(0, err.to_string());
                        //it must be infallable
                        MsgPackCodec::write(r_err, write_buff).unwrap();
                        write_buff.commit(crate::protocol::MSG_TYPE::ERR);
                    }
                }
            }
            Err(err) => {
                let write_buff = io_buf.write_buf();
                //we need to do this if read and write buffer are the same
                write_buff.clear();
                let r_err = RemoteError::new(0, err.to_string());
                //it must be infallable
                MsgPackCodec::write(r_err, write_buff).unwrap();
                write_buff.commit(crate::protocol::MSG_TYPE::ERR);
            }
        }
    }
}
