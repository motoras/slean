mod pingpong;
use dotenv::dotenv;
use hdrhistogram::Histogram;

use pingpong::*;
use slean::codec::{BincodeCodec, MsgPackCodec};
use slean::req::block::BlockingSleamService;
use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    dotenv().ok();
    env_logger::init();
    let mut handles = Vec::new();
    for _i in 0..1 {
        handles.push(std::thread::spawn(|| run_client()));
    }
    for handle in handles {
        handle.join().unwrap();
    }
}

fn run_client() {
    let mut hist = Histogram::<u64>::new_with_bounds(1, 1000 * 1000 * 60, 2).unwrap();
    let mut service = BlockingSleamService::<BincodeCodec, PingReq, PongRepl>::connect().unwrap();
    for _i in 0..1_000_000 {
        let start = crt_micros();
        let ping = PingReq::Ping(start);
        service.send(&ping).unwrap();
        let pong = service.receive().unwrap();
        let stop = crt_micros();
        match pong {
            PongRepl::Pong(pstart) => assert!(start == pstart),
        }
        let delta = (stop - start) as u64;
        hist.record(delta)
            .expect(&format!("{} should be in range", delta));
    }
    for v in hist.iter_quantiles(1) {
        println!(
            "{}'th percentile of data is {} with {} samples",
            v.percentile(),
            v.value_iterated_to(),
            v.count_at_value()
        );
    }
}

fn crt_micros() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_micros()
}
