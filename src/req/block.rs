use crate::error::RemoteError;
use crate::memo::SleamBuf;
use crate::protocol::{FrameDescriptor, MsgType, FRAME_DESC_SIZE_BYTES};
use crate::service::MsgPackCodec;

use byteorder::{LittleEndian, ReadBytesExt};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::io::ErrorKind;
use std::marker::PhantomData;
use std::net::TcpStream;
use std::time::Duration;

pub struct BlockingSleamService<Req, Repl>
where
    Req: Serialize,
    Repl: DeserializeOwned,
{
    req: PhantomData<Req>,
    repl: PhantomData<Repl>,
    buff: SleamBuf,
    tcp_stream: TcpStream,
}

impl<Req: Serialize, Repl: DeserializeOwned> BlockingSleamService<Req, Repl> {
    pub fn connect() -> Result<BlockingSleamService<Req, Repl>, std::io::Error> {
        let duration = Duration::from_secs(10);
        let buff = SleamBuf::default();
        let tcp_stream = TcpStream::connect_timeout(
            &std::net::SocketAddr::from(([127, 0, 0, 1], 2302)),
            duration,
        )?;
        tcp_stream.set_nonblocking(false)?;
        tcp_stream.set_nodelay(true)?;
        Ok(BlockingSleamService {
            buff,
            tcp_stream,
            req: PhantomData,
            repl: PhantomData,
        })
    }

    pub fn send(&mut self, req: &Req) -> Result<u32, std::io::Error> {
        self.buff.reset(FRAME_DESC_SIZE_BYTES);
        MsgPackCodec::write(req, &mut self.buff).unwrap();
        self.buff.commit(MsgType::Req);
        self.buff.copy_to(&mut self.tcp_stream)
    }

    pub fn receive(&mut self) -> Result<Repl, std::io::Error> {
        let desc = self.tcp_stream.read_u64::<LittleEndian>().unwrap();
        let fd: FrameDescriptor = desc.try_into().unwrap();
        self.buff.reset(0);
        self.buff.copy_from(&mut self.tcp_stream, fd.len()).unwrap();
        if fd.is_repl() {
            let repl: Repl = MsgPackCodec::read(&mut self.buff).unwrap();
            Ok(repl)
        } else if fd.is_err() {
            let err: RemoteError = MsgPackCodec::read(&mut self.buff).unwrap();
            dbg!(err);
            Err(ErrorKind::Other.into())
        } else {
            Err(ErrorKind::InvalidData.into())
        }
    }
}
