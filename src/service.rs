use crate::codec::Codec;
use crate::error::{RemoteError, SleanError};
use crate::memo::SleamBuf;
use crate::protocol::FRAME_DESC_SIZE_BYTES;
use log::error;
use std::marker::PhantomData;

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

pub trait ReplService {
    fn execute(&self, io_buf: &mut IoBuf);
}

pub struct OneReplyService<C, Req, Repl>
where
    C: Codec<Req> + Codec<Repl> + Codec<RemoteError>,
{
    worker: fn(&Req) -> Result<Repl, SleanError>,
    _codec: PhantomData<C>,
}

impl<C: Codec<Repl> + Codec<Req> + Codec<RemoteError>, Req, Repl> OneReplyService<C, Req, Repl> {
    pub fn new(worker: fn(&Req) -> Result<Repl, SleanError>) -> Self {
        OneReplyService {
            worker,
            _codec: PhantomData,
        }
    }
}

impl<C: Codec<Repl> + Codec<Req> + Codec<RemoteError>, Req, Repl> ReplService
    for OneReplyService<C, Req, Repl>
{
    fn execute(&self, io_buf: &mut IoBuf) {
        let read_buff = io_buf.read_buf();
        match <C as Codec<Req>>::read(read_buff) {
            Ok(req) => {
                let write_buff = io_buf.write_buf();
                //we need to do this if read and write buffer are the same
                write_buff.reset(FRAME_DESC_SIZE_BYTES);
                match (self.worker)(&req) {
                    Ok(repl) => match <C as Codec<Repl>>::write(&repl, write_buff) {
                        Ok(_) => {
                            write_buff.commit(crate::protocol::MsgType::Repl);
                        }
                        Err(err) => {
                            write_buff.reset(FRAME_DESC_SIZE_BYTES);

                            //it must be infallable
                            <C as Codec<RemoteError>>::write(&err.into(), write_buff).unwrap();
                            write_buff.commit(crate::protocol::MsgType::Err);
                        }
                    },
                    Err(err) => {
                        error!("Service error {}", err);
                        //it must be infallable
                        <C as Codec<RemoteError>>::write(&(0, err.to_string()), write_buff)
                            .unwrap();
                        write_buff.commit(crate::protocol::MsgType::Err);
                    }
                }
            }
            Err(err) => {
                error!("{}", err);
                let write_buff = io_buf.write_buf();
                //we need to do this if read and write buffer are the same
                write_buff.reset(FRAME_DESC_SIZE_BYTES);
                //it must be infallable
                <C as Codec<RemoteError>>::write(&err.into(), write_buff).unwrap();
                write_buff.commit(crate::protocol::MsgType::Err);
            }
        }
    }
}
