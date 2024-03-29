mod calc;
use calc::*;
use dotenv::dotenv;
use log::debug;
use slean::codec::MsgPackCodec;
use slean::req::block::BlockingSleamService;

fn main() {
    dotenv().ok();
    env_logger::init();
    let mut handles = Vec::new();
    for _i in 0..128 {
        handles.push(std::thread::spawn(|| run_client()));
    }
    for handle in handles {
        handle.join().unwrap();
    }
}

fn run_client() {
    let mut service =
        BlockingSleamService::<MsgPackCodec, CalcRequest, CalcReply>::connect().unwrap();

    let add_req = CalcRequest::Add(5, 7);
    let mul_req = CalcRequest::Mul(5, 7);

    for _i in 0..1_000_000 / 2 {
        service.send(&add_req).unwrap();
        let res: CalcReply = service.receive().unwrap();
        debug!("Got reply {:?}", res);
    }
    for _i in 0..1_000_000 / 2 {
        service.send(&mul_req).unwrap();
        let res: CalcReply = service.receive().unwrap();
        debug!("Got reply {:?}", res);
    }
}
