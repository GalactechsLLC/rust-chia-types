use crate::blockchain::vdf_info::VdfInfo;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ChallengeChainSubSlot {
    pub challenge_chain_end_of_slot_vdf: VdfInfo,
    pub new_sub_slot_iters: Option<u64>,
    pub new_difficulty: Option<u64>,
    pub infused_challenge_chain_sub_slot_hash: Option<String>,
    pub subepoch_summary_hash: Option<String>,
}
