use log::info;

use crate::memo::SleamBuf;
use crate::protocol::MSG_TYPE;
use crate::service::{MsgPackCodec, ReplService};

use crate::conn::Connection;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use log::{error, trace};
use serde::de::DeserializeOwned;
use serde::Serialize;
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
    write_buf: SleamBuf,
    tcp_stream: TcpStream,
}

impl<Req: Serialize, Repl: DeserializeOwned> BlockingSleamService<Req, Repl> {
    pub fn connect() -> Result<BlockingSleamService<Req, Repl>, std::io::Error> {
        let duration = Duration::from_secs(10);
        let write_buf = SleamBuf::default();
        let tcp_stream = TcpStream::connect_timeout(
            &std::net::SocketAddr::from(([127, 0, 0, 1], 2302)),
            duration,
        )?;
        tcp_stream.set_nonblocking(false)?;
        tcp_stream.set_nodelay(true)?;
        Ok(BlockingSleamService {
            write_buf,
            tcp_stream,
            req: PhantomData,
            repl: PhantomData,
        })
    }

    pub fn send(&mut self, req: &Req) -> Result<u32, std::io::Error> {
        self.write_buf.clear();
        MsgPackCodec::write(req, &mut self.write_buf).unwrap();
        self.write_buf.commit(MSG_TYPE::REQ);
        self.write_buf.copy_to(&mut self.tcp_stream)
    }

    pub fn receive(&mut self) -> Result<Repl, std::io::Error> {
        let _len = self.tcp_stream.read_u32::<LittleEndian>().unwrap();
        let repl: Repl = MsgPackCodec::read(&mut self.tcp_stream).unwrap();
        Ok(repl)
    }
}
