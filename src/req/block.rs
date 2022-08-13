use crate::codec::Codec;
use crate::error::RemoteError;
use crate::memo::SleamBuf;
use crate::protocol::{FrameDescriptor, MsgType, FRAME_DESC_SIZE_BYTES};

use byteorder::{LittleEndian, ReadBytesExt};
use std::io::ErrorKind;
use std::marker::PhantomData;
use std::net::TcpStream;
use std::time::Duration;

pub struct BlockingSleamService<C, Req, Repl>
where
    C: Codec<Req> + Codec<Repl> + Codec<RemoteError>,
{
    buff: SleamBuf,
    tcp_stream: TcpStream,
    _req: PhantomData<*const Req>,
    _repl: PhantomData<*const Repl>,
    _codec: PhantomData<*const C>,
}

impl<C: Codec<Repl> + Codec<Req> + Codec<RemoteError>, Req, Repl>
    BlockingSleamService<C, Req, Repl>
{
    pub fn connect() -> Result<BlockingSleamService<C, Req, Repl>, std::io::Error> {
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
            _req: PhantomData,
            _repl: PhantomData,
            _codec: PhantomData,
        })
    }

    pub fn send(&mut self, req: &Req) -> Result<u32, std::io::Error> {
        self.buff.reset(FRAME_DESC_SIZE_BYTES);
        <C as Codec<Req>>::write(req, &mut self.buff).unwrap();
        self.buff.commit(MsgType::Req);
        self.buff.copy_to(&mut self.tcp_stream)
    }

    pub fn receive(&mut self) -> Result<Repl, std::io::Error> {
        let desc = self.tcp_stream.read_u64::<LittleEndian>().unwrap();
        let fd: FrameDescriptor = desc.try_into().unwrap();
        self.buff.reset(0);
        self.buff.copy_from(&mut self.tcp_stream, fd.len()).unwrap();
        if fd.is_repl() {
            let repl: Repl = <C as Codec<Repl>>::read(&mut self.buff).unwrap();
            Ok(repl)
        } else if fd.is_err() {
            let err: RemoteError = <C as Codec<RemoteError>>::read(&mut self.buff).unwrap();
            dbg!(err);
            Err(ErrorKind::Other.into())
        } else {
            Err(ErrorKind::InvalidData.into())
        }
    }
}
