use std::io::{Read, Write};

use rmp_serde::encode::write;
use rmp_serde::from_read;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::error::{RemoteError, SleanError, SleanResult};
use crate::memo::SleamBuf;
use crate::protocol::FRAME_DESC_SIZE_BYTES;

#[repr(u8)]
pub enum IoBuf<'iobuff> {
    One(&'iobuff mut SleamBuf),
    Splited(&'iobuff mut SleamBuf, &'iobuff mut SleamBuf),
}

impl<'iobuff> IoBuf<'iobuff> {
    #[inline(always)]
    pub(crate) fn read_buf(&mut self) -> &mut SleamBuf {
        match self {
            IoBuf::One(read_buff) => read_buff,
            IoBuf::Splited(read_buff, _) => read_buff,
        }
    }

    #[inline(always)]
    pub(crate) fn write_buf(&mut self) -> &mut SleamBuf {
        match self {
            IoBuf::One(write_buff) => write_buff,
            IoBuf::Splited(_, write_buff) => write_buff,
        }
    }
}

pub struct MsgPackCodec {}

impl MsgPackCodec {
    pub fn read<Req>(reader: &mut impl Read) -> SleanResult<Req>
    where
        Req: DeserializeOwned,
    {
        from_read(reader).map_err(|err| SleanError::DecodingFailed(err.to_string()))
    }

    pub fn write<Repl>(repl: Repl, buff: &mut impl Write) -> SleanResult<()>
    where
        Repl: Serialize,
    {
        write(buff, &repl).map_err(|err| SleanError::EncodingFailed(err.to_string()))
    }
}

pub trait ReplService {
    fn execute(&self, io_buf: &mut IoBuf);
}

pub struct SimpleReplyService<Req, Repl> {
    pub worker: fn(Req) -> Result<Repl, SleanError>,
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
                write_buff.reset(FRAME_DESC_SIZE_BYTES);
                match (self.worker)(req) {
                    Ok(repl) => match MsgPackCodec::write(repl, write_buff) {
                        Ok(_) => {
                            write_buff.commit(crate::protocol::MsgType::Repl);
                        }
                        Err(err) => {
                            write_buff.reset(FRAME_DESC_SIZE_BYTES);
                            let r_err: RemoteError = err.into();
                            //it must be infallable
                            MsgPackCodec::write(r_err, write_buff).unwrap();
                            write_buff.commit(crate::protocol::MsgType::Err);
                        }
                    },
                    Err(err) => {
                        let r_err = RemoteError::new(0, err.to_string());
                        //it must be infallable
                        MsgPackCodec::write(r_err, write_buff).unwrap();
                        write_buff.commit(crate::protocol::MsgType::Err);
                    }
                }
            }
            Err(err) => {
                let write_buff = io_buf.write_buf();
                //we need to do this if read and write buffer are the same
                write_buff.reset(FRAME_DESC_SIZE_BYTES);
                let r_err: RemoteError = err.into();
                //it must be infallable
                MsgPackCodec::write(r_err, write_buff).unwrap();
                write_buff.commit(crate::protocol::MsgType::Err);
            }
        }
    }
}
