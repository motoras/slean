use dotenv::dotenv;
use log::info;
use slean::codec::TextCodec;
use slean::error::SleanResult;
use slean::{repl::ReplServer, service::OneReplyService};
fn echo(req: &String) -> SleanResult<String> {
    Ok(req.to_string())
}
fn main() {
    dotenv().ok();
    env_logger::init();
    let mut handles = Vec::new();
    for _i in 0..8 {
        handles.push(std::thread::spawn(run_server));
    }
    for handle in handles {
        handle.join().unwrap();
    }
}

fn run_server() {
    let service = OneReplyService::<TextCodec, String, String>::new(echo);
    let mut repl_server = ReplServer::new(service);
    info!("Starting the server");
    repl_server.server().unwrap();
}
