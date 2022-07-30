use log::info;

use crate::memo::TcpWriteBuff;
use crate::service::{MsgPackCodec, ReplService};

use crate::connection::Connection;
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
    write_buf: TcpWriteBuff,
    tcp_stream: TcpStream,
}

impl<Req: Serialize, Repl: DeserializeOwned> BlockingSleamService<Req, Repl> {
    pub fn connect() -> Result<BlockingSleamService<Req, Repl>, std::io::Error> {
        let duration = Duration::from_secs(10);
        let write_buf = TcpWriteBuff::default();
        let tcp_stream = TcpStream::connect_timeout(
            &std::net::SocketAddr::from(([127, 0, 0, 1], 2302)),
            duration,
        )?;
        tcp_stream.set_nonblocking(false)?;
        Ok(BlockingSleamService {
            write_buf,
            tcp_stream,
            req: PhantomData,
            repl: PhantomData,
        })
    }

    pub fn send(&mut self, req: &Req) -> Result<(), std::io::Error> {
        MsgPackCodec::write(req, &mut self.write_buf).unwrap();
        self.write_buf.send(&mut self.tcp_stream)
    }

    pub fn receive(&mut self) -> Result<Repl, std::io::Error> {
        let _len = self.tcp_stream.read_u32::<LittleEndian>().unwrap();
        let repl: Repl = MsgPackCodec::read(&mut self.tcp_stream).unwrap();
        Ok(repl)
    }
}
