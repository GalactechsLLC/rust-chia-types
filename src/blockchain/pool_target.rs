use crate::blockchain::sized_bytes::Bytes32;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct PoolTarget {
    pub max_height: u32,
    pub puzzle_hash: Bytes32,
}
