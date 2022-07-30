use log::info;

use crate::memo::TcpWriteBuff;
use crate::service::ReplService;

use crate::connection::Connection;
use byteorder::{LittleEndian, WriteBytesExt};
use log::{error, trace};
use mio::net::{TcpListener, TcpStream};
use mio::{Events, Interest, Poll, Token};
use std::io::ErrorKind;
use std::time::Duration;

use std::io::Error as IoError;

pub struct BlockingReqService {
    write_buf: TcpWriteBuff,
    tcp_Stream: TcpStream,
}

impl BlockingReqService {
    pub fn connect() -> BlockingReqService {
        let duration = Duration::from_secs(10);
        let mut write_buf = TcpWriteBuff::default();
        let duration = Duration::from_secs(10);
        match TcpStream::connect_timeout(
            &std::net::SocketAddr::from(([127, 0, 0, 1], 2302)),
            duration,
        ) {
            Ok(mut stream) => {}
            Err(e) => {
                println!("Failed to connect: {}", e);
                return e;
            }
        }
        todo!("Working on");
    }
    pub fn connect2() {
        let mut write_buf = TcpWriteBuff::default();
        let mut read_buf: [u8; 128] = [0u8; 128];
        match TcpStream::connect_timeout(
            &std::net::SocketAddr::from(([127, 0, 0, 1], 2302)),
            duration,
        ) {
            Ok(mut stream) => {
                for _i in 0..1_000_00 / 2 {
                    //info!("Sending request  {:?}", add_req);
                    MsgPackCodec::write(&add_req, &mut write_buf).unwrap();
                    write_buf.send(&mut stream).unwrap();
                    let len = stream.read_u32::<LittleEndian>().unwrap();
                    stream.read(&mut read_buf[0..len as usize]).unwrap();
                    let res: CalcReply =
                        MsgPackCodec::read(&mut &read_buf[0..len as usize]).unwrap();
                    debug!("Got reply {:?}", res);
                }
                for _i in 0..1_000_00 / 2 {
                    //info!("Sending request  {:?}", add_req);
                    MsgPackCodec::write(&mul_req, &mut write_buf).unwrap();
                    write_buf.send(&mut stream).unwrap();
                    let len = stream.read_u32::<LittleEndian>().unwrap();
                    stream.read(&mut read_buf[0..len as usize]).unwrap();
                    let res: CalcReply =
                        MsgPackCodec::read(&mut &read_buf[0..len as usize]).unwrap();
                    debug!("Got reply {:?}", res);
                }
            }
            Err(e) => {
                println!("Failed to connect: {}", e);
                return;
            }
        }
    }
}

// // Some tokens to allow us to identify which event is for which socket.
// const SERVER: Token = Token(1 << 29);

// use socket2::{Domain, Protocol, Socket, Type};
// use std::net::SocketAddr;

// pub struct ReqServer {
//     write_buff: TcpWriteBuff,
// }

// impl ReqServer {
//     pub fn new() -> Self {
//         ReqServer {
//             write_buff: TcpWriteBuff::default(),
//         }
//     }

//     fn connect(socket: &mut Socket, addr: SocketAddr) -> Result<(), IoError> {
//         socket.set_nonblocking(true)?;
//         socket.connect_timeout(&addr.into(), Duration::from_secs(1))?;
//         Ok(())
//     }

//     pub fn server(&mut self) -> Result<(), std::io::Error> {
//         // Setup the server socket.
//         let addr: SocketAddr = "127.0.0.1:2302".parse().unwrap();
//         let mut socket = Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP))?;
//         ReqServer::connect(&mut socket, addr)?;
//         let mut server: TcpListener = TcpListener::from_std(socket.into());

//         let mut poll = Poll::new()?;
//         let mut events = Events::with_capacity(128);

//         poll.registry().register(
//             &mut server,
//             SERVER,
//             Interest::READABLE.add(Interest::WRITABLE),
//         )?;

//         loop {
//             poll.poll(&mut events, None)?;
//             for event in events.iter() {
//                 trace!("Event {:?}", &event);
//                 match event.token() {
//                     SERVER => {
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
//                 }
//             }
//         }
//     }
// }
