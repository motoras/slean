mod calc;
use calc::*;
use dotenv::dotenv;
use log::info;
use sleam::{repl::ReplServer, service::SimpleReplyService};
fn calculator(req: CalcRequest) -> CalcReply {
    match req {
        CalcRequest::Add(x, y) => CalcReply::Sum(x + y),
        CalcRequest::Mul(x, y) => CalcReply::Product(x * y),
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
    let service = SimpleReplyService { worker: calculator };
    let mut repl_server = ReplServer::new(service);
    info!("Starting the server");
    repl_server.server().unwrap();
}
