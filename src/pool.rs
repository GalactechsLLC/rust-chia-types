use crate::blockchain::coin_spend::CoinSpend;
use crate::blockchain::sized_bytes::{Bytes32, Bytes48};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Farmer {
    pub launcher_id: Bytes32,
    pub p2_singleton_puzzle_hash: Bytes32,
    pub delay_time: u64,
    pub delay_puzzle_hash: Bytes32,
    pub authentication_public_key: Bytes48,
    pub singleton_tip: String,
    pub singleton_tip_state: String,
    pub balance: u64,
    pub points: u64,
    pub difficulty: u64,
    pub payout_instructions: String,
    pub is_pool_member: bool,
    pub joined: u64,
    pub modified: u64,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct PoolState {
    pub version: u8,
    pub state: u8,
    pub target_puzzle_hash: Bytes32,
    pub owner_pubkey: Bytes48,
    pub pool_url: String,
    pub relative_lock_height: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SingletonState {
    pub saved_solution: CoinSpend,
    pub saved_state: PoolState,
    pub last_not_null_state: PoolState,
}
