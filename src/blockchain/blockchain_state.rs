use crate::blockchain::block_record::BlockRecord;
use crate::blockchain::sized_bytes::Bytes32;
use crate::blockchain::sync::Sync;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct MinMempoolFees {
    pub cost_5000000: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BlockchainState {
    pub peak: Option<BlockRecord>,
    pub genesis_challenge_initialized: bool,
    pub sync: Sync,
    pub difficulty: u64,
    pub sub_slot_iters: u64,
    pub space: u128,
    pub mempool_size: u64,
    pub mempool_cost: u64,
    pub mempool_min_fees: MinMempoolFees,
    pub mempool_max_total_cost: u64,
    pub block_max_cost: u64,
    pub node_id: Bytes32,
}
