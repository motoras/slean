use serde_derive::{Deserialize, Serialize};
#[derive(Debug, Serialize, Deserialize)]
pub enum CalcRequest {
    Add(usize, usize),
    Mul(usize, usize),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum CalcReply {
    Sum(usize),
    Product(usize),
}
