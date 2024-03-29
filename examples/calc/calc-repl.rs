mod calc;

use calc::*;
use dotenv::dotenv;
use log::info;
use slean::codec::MsgPackCodec;
use slean::error::SleanResult;
use slean::{repl::ReplServer, service::OneReplyService};
fn calculator(req: &CalcRequest) -> SleanResult<CalcReply> {
    info!("Request is {:?}", &req);
    match req {
        CalcRequest::Add(x, y) => Ok(CalcReply::Sum(x + y)),
        CalcRequest::Mul(x, y) => Ok(CalcReply::Product(x * y)),
    }
}
fn main() {
    dotenv().ok();
    env_logger::init();
    let mut handles = Vec::new();
    for _i in 0..2 {
        handles.push(std::thread::spawn(run_server));
    }
    for handle in handles {
        handle.join().unwrap();
    }
}

fn run_server() {
    let service = OneReplyService::<MsgPackCodec, _, _>::new(calculator);
    let mut repl_server = ReplServer::new(service);
    info!("Starting the server");
    repl_server.server().unwrap();
}
