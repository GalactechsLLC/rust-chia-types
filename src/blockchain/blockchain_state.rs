use crate::blockchain::block_record::BlockRecord;
use crate::blockchain::sync::Sync;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct BlockchainState {
    pub difficulty: u64,
    pub sub_slot_iters: u64,
    pub mempool_size: u32,
    pub genesis_challenge_initialized: bool,
    pub space: u128,
    pub peak: BlockRecord,
    pub sync: Sync,
}
