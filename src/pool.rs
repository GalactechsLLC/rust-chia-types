use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct PoolState {
    pub version: u8,
    pub state: u8,
    pub target_puzzle_hash: String,
    pub owner_pubkey: String,
    pub pool_url: String,
    pub relative_lock_height: u32,
}
