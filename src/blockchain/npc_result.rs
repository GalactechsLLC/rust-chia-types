use crate::blockchain::npc::NPC;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct NPCResult {
    pub error: Option<u16>,
    pub clvm_cost: u64,
    pub npc_list: Vec<NPC>,
}
