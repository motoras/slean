use log::info;

use crate::memo::SleamBuf;
use crate::service::ReplService;

use crate::conn::Connection;
use std::io::ErrorKind;

use log::{error, trace};
use mio::net::{TcpListener, TcpStream};
use mio::{Events, Interest, Poll, Token};
use std::collections::HashMap;

use std::io::Error as IoError;

// Some tokens to allow us to identify which event is for which socket.
const SERVER: Token = Token(1 << 29);

use socket2::{Domain, Protocol, Socket, Type};
use std::net::SocketAddr;

pub struct ReplServer<RS: ReplService> {
    service: RS,
    buffer: SleamBuf,
}

impl<RS: ReplService> ReplServer<RS> {
    pub fn new(service: RS) -> Self {
        ReplServer {
            service,
            buffer: SleamBuf::default(),
        }
    }

    fn bind(socket: &mut Socket, addr: SocketAddr) -> Result<(), IoError> {
        socket.set_reuse_address(true)?;
        socket.set_reuse_port(true)?;
        socket.set_nonblocking(true)?;
        socket.set_nodelay(true)?;
        socket.bind(&addr.into())?;
        socket.listen(128)?;
        Ok(())
    }

    pub fn server(&mut self) -> Result<(), std::io::Error> {
        // Setup the server socket.
        let addr: SocketAddr = "127.0.0.1:2302".parse().unwrap();
        let mut socket = Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP))?;
        Self::bind(&mut socket, addr).unwrap();
        let mut server: TcpListener = TcpListener::from_std(socket.into());

        let mut poll = Poll::new()?;
        let mut events = Events::with_capacity(128);

        poll.registry()
            .register(&mut server, SERVER, Interest::READABLE)?;

        let mut connections = HashMap::new();
        // Unique token for each incoming connection.
        let mut crt_token = 1;
        info!("Start listening");
        loop {
            poll.poll(&mut events, None)?;
            for event in events.iter() {
                trace!("Event {:?}", &event);
                match event.token() {
                    SERVER => loop {
                        if event.is_writable() {
                            continue;
                        }
                        match server.accept() {
                            Ok((mut stream, address)) => {
                                let token = Token(crt_token);
                                crt_token += 1;
                                poll.registry()
                                    .register(&mut stream, token, Interest::READABLE)?;
                                let connection = Connection::new(stream, &self.service);
                                connections.insert(token, connection);
                                info!("Connection registered: {} {:?}", address, &token);
                            }
                            Err(e) if e.kind() == ErrorKind::WouldBlock => {
                                //info!("Failed connection accept:");
                                break;
                            }
                            Err(err) => {
                                error!("Error accepting connection... {}", err);
                                break;
                                //return Err(Box::new(err));
                            }
                        }
                    },
                    token => {
                        if event.is_error() {
                            info!("1 Connection closed {:?} ", &token);
                            if let Some(mut conn) = connections.remove(&token) {
                                poll.registry().deregister(&mut conn.stream).unwrap();
                            }
                            continue;
                        }
                        if event.is_read_closed() || event.is_write_closed() {
                            info!("2 Connection closed {:?}", &token);
                            if let Some(mut conn) = connections.remove(&token) {
                                poll.registry().deregister(&mut conn.stream).unwrap();
                            }
                            continue;
                        }
                        if event.is_writable() {
                            match connections.get_mut(&token) {
                                Some(conn) => {
                                    if let Err(err) = conn.on_write(&mut self.buffer) {
                                        error!("Error writing into connection {}", err);
                                        connections.remove(&token);
                                    } else {
                                        let write_pend = conn.is_write_pending();
                                        register_interest(
                                            &mut conn.stream,
                                            &poll,
                                            token,
                                            write_pend,
                                        );
                                    }
                                }
                                None => {
                                    info!("No connection for {:?}", &token);
                                    continue;
                                }
                            }
                        }
                        if event.is_readable() {
                            info!("Got READ message from {:?}", &token);
                            match connections.get_mut(&token) {
                                Some(conn) => {
                                    if !conn.is_write_pending() {
                                        if let Err(err) = conn.on_read(&mut self.buffer) {
                                            error!("Error reading from connection {}", err);
                                            connections.remove(&token);
                                        } else {
                                            let write_pend = conn.is_write_pending();
                                            register_interest(
                                                &mut conn.stream,
                                                &poll,
                                                token,
                                                write_pend,
                                            );
                                        }
                                    } else {
                                        info!("Won't read from {:?} until we can write previous replies", &token);
                                    }
                                }
                                None => {
                                    info!("No connection for {:?}", &token);
                                    continue;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[inline(always)]
fn register_interest(stream: &mut TcpStream, poll: &Poll, token: Token, write_pending: bool) {
    let interest = Interest::READABLE;
    if write_pending {
        interest.add(Interest::WRITABLE);
    }
    poll.registry().reregister(stream, token, interest).unwrap();
}
