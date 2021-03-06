use crate::blockchain::coin::Coin;
use crate::blockchain::npc_result::NPCResult;
use crate::blockchain::spend_bundle::SpendBundle;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct MemPoolItem {
    pub spend_bundle: SpendBundle,
    pub fee: u64,
    pub cost: u64,
    pub npc_result: NPCResult,
    pub spend_bundle_name: String,
    pub program: String,
    pub additions: Vec<Coin>,
    pub removals: Vec<Coin>,
}
