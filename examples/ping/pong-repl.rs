mod pingpong;
use dotenv::dotenv;
use log::info;
use pingpong::*;
use slean::codec::BincodeCodec;
use slean::error::SleanResult;
use slean::{repl::ReplServer, service::OneReplyService};
fn pingpong(req: &PingReq) -> SleanResult<PongRepl> {
    match *req {
        PingReq::Ping(ts) => Ok(PongRepl::Pong(ts)),
    }
}
fn main() {
    dotenv().ok();
    env_logger::init();
    let mut handles = Vec::new();
    for _i in 0..1 {
        handles.push(std::thread::spawn(|| run_server()));
    }
    for handle in handles {
        handle.join().unwrap();
    }
}

fn run_server() {
    let service = OneReplyService::<BincodeCodec, _, _>::new(pingpong);
    let mut repl_server = ReplServer::new(service);
    info!("Starting the server");
    repl_server.server().unwrap();
}
