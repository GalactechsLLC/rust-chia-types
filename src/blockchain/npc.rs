use crate::blockchain::sized_bytes::Bytes32;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct NPC {
    pub coin_name: Bytes32,
    pub puzzle_hash: Bytes32,
    pub conditions: Vec<(u8, Vec<(u8, String)>)>,
}
