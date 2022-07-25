use log::info;
use std::net::TcpStream;
use std::sync::Arc;
use std::{io::Write, time::Duration};

use crate::connection;

use connection::Connection;
use std::io::ErrorKind;

use log::{error, trace};
use mio::net::TcpListener;
use mio::{Events, Interest, Poll, Token};
use std::collections::HashMap;

use std::io::Error as IoError;

// Some tokens to allow us to identify which event is for which socket.
const SERVER: Token = Token(1 << 29);

use socket2::{Domain, Protocol, Socket, Type};
use std::net::SocketAddr;

struct ReplServer {
    server: TcpListener,
}

impl ReplServer {
    fn bind(socket: Socket, addr: SocketAddr) -> Result<(), IoError> {
        socket.set_reuse_address(true)?;
        socket.set_reuse_port(true)?;
        socket.set_nonblocking(true)?;
        socket.bind(&addr.into())?;
        socket.listen(128)?;
        Ok(())
    }

    fn on_receive(msg: &[u8], repl: Replier) {}
}

fn server() -> Result<(), ServerError> {
    // Setup the server socket.
    let addr: SocketAddr = "127.0.0.1:2302".parse()?;

    let socket = Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP))?;

    socket.set_reuse_address(true)?;
    socket.set_reuse_port(true)?;
    socket.set_nonblocking(true)?;
    socket.bind(&addr.into())?;
    socket.listen(128)?;
    let mut server: TcpListener = TcpListener::from_std(socket.into());

    let mut poll = Poll::new()?;
    let mut events = Events::with_capacity(128);

    poll.registry()
        .register(&mut server, SERVER, Interest::READABLE)?;

    let mut connections = HashMap::new();
    // Unique token for each incoming connection.
    let mut crt_token = 1;
    let mut data = [0 as u8; 1000 * 1000 * 2]; // the read buffer,  must be alocated once..
    info!("Start listening");
    loop {
        poll.poll(&mut events, None)?;
        for event in events.iter() {
            trace!("Event {:?}", &event);
            match event.token() {
                SERVER => loop {
                    match server.accept() {
                        Ok((mut stream, address)) => {
                            let token = Token(crt_token);
                            crt_token += 1;
                            poll.registry().register(
                                &mut stream,
                                token,
                                Interest::READABLE.add(Interest::WRITABLE),
                            )?;
                            let connection = Connection::new(token, stream);
                            connections.insert(token, connection);
                            //info!("Connection registered: {} {:?}", address, &token);
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
                        //info!("1 Connection closed {:?} ", &token);
                        if let Some(mut conn) = connections.remove(&token) {
                            poll.registry().deregister(&mut conn.stream).unwrap();
                        }
                        continue;
                    }
                    if event.is_read_closed() || event.is_write_closed() {
                        //info!("2 Connection closed {:?}", &token);
                        if let Some(mut conn) = connections.remove(&token) {
                            poll.registry().deregister(&mut conn.stream).unwrap();
                        }
                        continue;
                    }

                    if event.is_readable() {
                        match connections.get_mut(&token) {
                            Some(conn) => {
                                if let Err(err) = conn.on_read(&mut data[..]) {
                                    error!("Error reading from connection {}", err);
                                    connections.remove(&token);
                                }
                            }
                            None => {
                                info!("No connection for {:?}", &token);
                                continue;
                            }
                        }
                    }
                    if event.is_writable() {}
                }
            }
        }
    }
}
