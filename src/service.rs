use std::io::{Error, Read, Write};

use rmp_serde::encode::write;
use rmp_serde::from_read;
use serde::de::DeserializeOwned;
use serde::Serialize;

pub trait ReplService {
    fn execute(&self, sinp: &mut impl Read, sout: &mut impl Write) -> Result<(), Error>;
}

pub struct SimpleReplyService<Req, Repl> {
    pub worker: fn(Req) -> Repl,
}

impl<Req, Repl> ReplService for SimpleReplyService<Req, Repl>
where
    Req: DeserializeOwned,
    Repl: Serialize,
{
    fn execute(&self, sinp: &mut impl Read, sout: &mut impl Write) -> Result<(), Error> {
        let req = MsgPackCodec::read(sinp).unwrap();
        let repl = (self.worker)(req);
        MsgPackCodec::write(repl, sout).unwrap();
        Ok(())
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
