mod calc;
use dotenv::dotenv;

use slean::codec::TextCodec;
use slean::req::block::BlockingSleamService;

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
    let mut service = BlockingSleamService::<TextCodec, String, String>::connect().unwrap();
    let hello = "Hello world".to_string();
    for _i in 0..1_000_000 {
        service.send(&hello).unwrap();
        let res = service.receive().unwrap();
        assert_eq!(hello, res);
    }
}
