//will revisite this later
use log::info;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::memo::TcpWriteBuff;
use crate::service::{MsgPackCodec, ReplService};

use crate::conn::Connection;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use log::{error, trace};
use mio::net::{TcpListener, TcpStream};
use mio::{Events, Interest, Poll, Token};
use std::io::ErrorKind;
use std::marker::PhantomData;
use std::time::Duration;

use std::io::Error as IoError;

// Some tokens to allow us to identify which event is for which socket.
const REQ: Token = Token(1 << 29);

use socket2::{Domain, Protocol, Socket, Type};
use std::net::SocketAddr;

pub struct ReqServer<Req, Repl>
where
    Req: Serialize,
    Repl: DeserializeOwned,
{
    write_buf: TcpWriteBuff,
    req: PhantomData<Req>,
    repl: PhantomData<Repl>,
}

// impl<Req: Serialize, Repl: DeserializeOwned> ReqServer<Req, Repl> {
//     pub fn new() -> Self {
//         ReqServer {
//             write_buf: TcpWriteBuff::default(),
//             req: PhantomData,
//             repl: PhantomData,
//         }
//     }

//     pub fn connect(&mut self) -> Result<(), std::io::Error> {
//         // Setup the client socket.
//         let addr: SocketAddr = "127.0.0.1:2302".parse().unwrap();
//         let mut socket = Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP))?;
//         socket.set_nonblocking(true)?;
//         socket.connect_timeout(&addr.into(), Duration::from_secs(1))?;
//         let mut tcp_stream: TcpStream = TcpStream::from_std(socket.into());

//         let mut poll = Poll::new()?;
//         let mut events = Events::with_capacity(128);

//         poll.registry().register(
//             &mut tcp_stream,
//             REQ,
//             Interest::READABLE.add(Interest::WRITABLE),
//         )?;

//         loop {
//             poll.poll(&mut events, None)?;
//             for event in events.iter() {
//                 trace!("Event {:?}", &event);
//                 match event.token() {
//                     REQ => {
//                         if event.is_error() {
//                             continue;
//                         }
//                         if event.is_read_closed() || event.is_write_closed() {
//                             info!("2 Connection closed");
//                         }

//                         if event.is_readable() {
//                             // Some(conn) => {
//                             //     if let Err(err) = conn.on_read(&mut self.write_buff) {
//                             //         error!("Error reading from connection {}", err);
//                             //         connections.remove(&token);
//                             //     } else {
//                             //         poll.registry()
//                             //             .reregister(&mut conn.stream, token, Interest::READABLE)
//                             //             .unwrap();
//                             //     }
//                             // }
//                             // None => {
//                             //     info!("No connection for {:?}", &token);
//                             //     continue;
//                             // }
//                         }
//                         if event.is_writable() {}
//                     }
//                     _ => {}
//                 }
//             }
//         }
//     }

//     pub fn send(&mut self, tcp_stream: TcpStream, req: &Req) -> Result<(), std::io::Error> {
//         MsgPackCodec::write(req, &mut self.write_buf).unwrap();
//         self.write_buf.send(&mut tcp_stream)
//     }

//     pub fn receive(&mut self, tcp_stream: &mut TcpStream) -> Result<Repl, std::io::Error> {
//         let _len = tcp_stream.read_u32::<LittleEndian>().unwrap();
//         let repl: Repl = MsgPackCodec::read(&mut tcp_stream).unwrap();
//         Ok(repl)
//     }
// }
