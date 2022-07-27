mod calc;
use byteorder::ReadBytesExt;
use byteorder::{LittleEndian, WriteBytesExt};
use calc::*;
use dotenv::dotenv;
use log::debug;
use sleam::memo::TcpWriteBuff;
use sleam::service::MsgPackCodec;
use std::net::TcpStream;
use std::{io::Read, io::Write, time::Duration};
fn main() {
    dotenv().ok();
    env_logger::init();
    let mut handles = Vec::new();
    for _i in 0..4 {
        handles.push(std::thread::spawn(|| run_client()));
    }
    for handle in handles {
        handle.join().unwrap();
    }
}

fn run_client() {
    let duration = Duration::from_secs(100);
    let add_req = CalcRequest::Add(5, 7);
    let mul_req = CalcRequest::Mul(5, 7);
    let mut write_buf = TcpWriteBuff::default();
    let mut read_buf: [u8; 128] = [0u8; 128];
    match TcpStream::connect_timeout(
        &std::net::SocketAddr::from(([127, 0, 0, 1], 2302)),
        duration,
    ) {
        Ok(mut stream) => {
            for _i in 0..1_000_000 / 2 {
                //info!("Sending request  {:?}", add_req);
                MsgPackCodec::write(&add_req, &mut write_buf).unwrap();
                write_buf.send(&mut stream).unwrap();
                let len = stream.read_u32::<LittleEndian>().unwrap();
                stream.read(&mut read_buf[0..len as usize]).unwrap();
                let res: CalcReply = MsgPackCodec::read(&mut &read_buf[0..len as usize]).unwrap();
                debug!("Got reply {:?}", res);
            }
            for _i in 0..1_000_000 / 2 {
                //info!("Sending request  {:?}", add_req);
                MsgPackCodec::write(&mul_req, &mut write_buf).unwrap();
                write_buf.send(&mut stream).unwrap();
                let len = stream.read_u32::<LittleEndian>().unwrap();
                stream.read(&mut read_buf[0..len as usize]).unwrap();
                let res: CalcReply = MsgPackCodec::read(&mut &read_buf[0..len as usize]).unwrap();
                debug!("Got reply {:?}", res);
            }
        }
        Err(e) => {
            println!("Failed to connect: {}", e);
            return;
        }
    }
}
