use hex::{decode, encode};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::error::Error;
//#[serde(deserialize_with = "crate::")]

pub fn u64_to_bytes(v: u64) -> Vec<u8> {
    let mut rtn = Vec::new();
    if v.leading_zeros() == 0 {
        rtn.push(u8::MIN);
        let ary = v.to_be_bytes();
        rtn.extend_from_slice(&ary);
        rtn
    } else {
        let mut trim: bool = true;
        for b in v.to_be_bytes() {
            if trim {
                if b == u8::MIN {
                    continue;
                } else {
                    rtn.push(b);
                    trim = false;
                }
            } else {
                rtn.push(b);
            }
        }
        rtn
    }
}

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

#[derive(Serialize, Deserialize, Debug)]
pub struct BlockRecord {
    pub challenge_block_info_hash: String,
    pub farmer_puzzle_hash: String,
    pub header_hash: String,
    pub pool_puzzle_hash: String,
    pub prev_hash: String,
    pub prev_transaction_block_hash: Option<String>,
    pub reward_infusion_new_challenge: String,
    pub finished_challenge_slot_hashes: Option<Vec<String>>,
    pub finished_infused_challenge_slot_hashes: Option<Vec<String>>,
    pub finished_reward_slot_hashes: Option<Vec<String>>,
    pub height: u32,
    pub prev_transaction_block_height: u32,
    pub signage_point_index: u8,
    pub deficit: u8,
    pub fees: Option<u64>,
    pub sub_slot_iters: u64,
    pub timestamp: Option<u64>,
    pub required_iters: u64,
    pub total_iters: u128,
    pub weight: u128,
    pub challenge_vdf_output: VdfOutput,
    pub infused_challenge_vdf_output: Option<VdfOutput>,
    pub overflow: bool,
    pub reward_claims_incorporated: Option<Vec<Coin>>,
    pub sub_epoch_summary_included: Option<SubEpochSummary>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Coin {
    pub amount: u64,
    pub parent_coin_info: String,
    pub puzzle_hash: String,
}
impl Coin {
    pub async fn name(&self) -> Result<String, Box<dyn Error>> {
        Ok(encode(self.hash().await.unwrap()))
    }

    fn prep_hex_str(&self, to_fix: &String) -> String {
        let rtn: String;
        if to_fix.starts_with("0x") {
            rtn = to_fix.strip_prefix("0x").unwrap().to_string();
        } else {
            rtn = to_fix.to_string();
        }
        rtn
    }

    pub async fn hash(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut to_hash: Vec<u8> = Vec::new();
        to_hash.extend(decode(self.prep_hex_str(&self.parent_coin_info))?);
        to_hash.extend(decode(self.prep_hex_str(&self.puzzle_hash))?);
        to_hash.extend(u64_to_bytes(self.amount));
        let mut hasher: Sha256 = Sha256::new();
        hasher.update(to_hash);
        Ok(hasher.finalize().to_vec())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CoinRecord {
    pub coin: Coin,
    pub confirmed_block_index: u32,
    pub spent_block_index: u32,
    pub timestamp: u64,
    pub coinbase: bool,
    pub spent: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CoinSpend {
    pub coin: Coin,
    pub puzzle_reveal: String,
    pub solution: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChallengeBlockInfo {
    pub proof_of_space: ProofOfSpace,
    pub challenge_chain_sp_vdf: Option<VdfInfo>,
    pub challenge_chain_sp_signature: String,
    pub challenge_chain_ip_vdf: VdfInfo,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChallengeChainSubSlot {
    pub challenge_chain_end_of_slot_vdf: VdfInfo,
    pub new_sub_slot_iters: Option<u64>,
    pub new_difficulty: Option<u64>,
    pub infused_challenge_chain_sub_slot_hash: Option<String>,
    pub subepoch_summary_hash: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Foliage {
    pub foliage_transaction_block_hash: Option<String>,
    pub prev_block_hash: String,
    pub reward_block_hash: String,
    pub foliage_block_data_signature: String,
    pub foliage_transaction_block_signature: Option<String>,
    pub foliage_block_data: FoliageBlockData,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FoliageBlockData {
    pub extension_data: String,
    pub farmer_reward_puzzle_hash: String,
    pub unfinished_reward_block_hash: String,
    pub pool_signature: Option<String>,
    pub pool_target: PoolTarget,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FoliageTransactionBlock {
    pub additions_root: String,
    pub filter_hash: String,
    pub prev_transaction_block_hash: String,
    pub removals_root: String,
    pub transactions_info_hash: String,
    pub timestamp: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FullBlock {
    pub challenge_chain_ip_proof: VdfProof,
    pub challenge_chain_sp_proof: VdfProof,
    pub infused_challenge_chain_ip_proof: Option<VdfProof>,
    pub reward_chain_ip_proof: VdfProof,
    pub reward_chain_sp_proof: Option<VdfProof>,
    pub foliage: Foliage,
    pub foliage_transaction_block: Option<FoliageTransactionBlock>,
    pub transactions_generator: Option<String>,
    pub transactions_generator_ref_list: Vec<u32>,
    pub finished_sub_slots: Vec<SubSlotBundle>,
    pub reward_chain_block: RewardChainBlock,
    pub transactions_info: Option<TransactionsInfo>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InfusedChallengeChainSubSlot {
    pub infused_challenge_chain_end_of_slot_vdf: VdfInfo,
}

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

#[derive(Serialize, Deserialize, Debug)]
pub struct NetworkInfo {
    pub network_name: String,
    pub network_prefix: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NPC {
    pub coin_name: String,
    pub puzzle_hash: String,
    pub conditions: Vec<(u8, Vec<(u8, String)>)>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NPCResult {
    pub error: Option<u16>,
    pub clvm_cost: u64,
    pub npc_list: Vec<NPC>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PendingPayment {
    pub puzzle_hash: String,
    pub amount: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProofOfSpace {
    pub challenge: String,
    pub pool_contract_puzzle_hash: Option<String>,
    pub plot_public_key: String,
    pub pool_public_key: Option<String>,
    pub proof: String,
    pub size: u8,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PoolTarget {
    pub max_height: u32,
    pub puzzle_hash: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RewardChainBlock {
    pub pos_ss_cc_challenge_hash: String,
    pub challenge_chain_sp_signature: String,
    pub reward_chain_sp_signature: String,
    pub challenge_chain_sp_vdf: Option<VdfInfo>,
    pub infused_challenge_chain_ip_vdf: Option<VdfInfo>,
    pub challenge_chain_ip_vdf: VdfInfo,
    pub reward_chain_ip_vdf: VdfInfo,
    pub reward_chain_sp_vdf: Option<VdfInfo>,
    pub height: u64,
    pub signage_point_index: u8,
    pub total_iters: u128,
    pub weight: u128,
    pub is_transaction_block: bool,
    pub proof_of_space: ProofOfSpace,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RewardChainBlockUnfinished {
    pub total_iters: u128,
    pub signage_point_index: u8,
    pub pos_ss_cc_challenge_hash: String,
    pub proof_of_space: ProofOfSpace,
    pub challenge_chain_sp_vdf: Option<VdfInfo>,
    pub challenge_chain_sp_signature: String,
    pub reward_chain_sp_vdf: Option<VdfInfo>,
    pub reward_chain_sp_signature: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RewardChainSubSlot {
    pub end_of_slot_vdf: VdfInfo,
    pub challenge_chain_sub_slot_hash: String,
    pub infused_challenge_chain_sub_slot_hash: Option<String>,
    pub deficit: u8,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SignagePoint {
    pub cc_vdf: VdfInfo,
    pub cc_proof: VdfProof,
    pub rc_vdf: VdfInfo,
    pub rc_proof: VdfProof,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SignagePointOrEOS {
    pub signage_point: SignagePoint,
    pub eos: SubSlotBundle,
    pub time_received: f64,
    pub reverted: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SpendBundle {
    pub coin_spends: Vec<CoinSpend>,
    pub aggregated_signature: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SubEpochSummary {
    pub prev_subepoch_summary_hash: String,
    pub reward_chain_hash: String,
    pub num_blocks_overflow: u8,
    pub new_difficulty: Option<u64>,
    pub new_sub_slot_iters: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SubSlotBundle {
    pub challenge_chain: ChallengeChainSubSlot,
    pub infused_challenge_chain: Option<InfusedChallengeChainSubSlot>,
    pub reward_chain: RewardChainSubSlot,
    pub proofs: SubSlotProofs,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SubSlotProofs {
    pub challenge_chain_slot_proof: VdfProof,
    pub infused_challenge_chain_slot_proof: Option<VdfProof>,
    pub reward_chain_slot_proof: VdfProof,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Sync {
    pub sync_mode: bool,
    pub synced: bool,
    pub sync_tip_height: u32,
    pub sync_progress_height: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TransactionPeer {
    pub peer: String,
    pub error: String,
    pub status: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TransactionRecord {
    pub confirmed_at_height: u32,
    pub sent: u32,
    pub wallet_id: u32,
    #[serde(alias = "type")]
    pub wallet_type: WalletType,
    pub created_at_time: u64,
    pub amount: u64,
    pub fee_amount: u64,
    pub to_puzzle_hash: String,
    pub trade_id: u64,
    pub name: u64,
    pub confirmed: bool,
    pub spend_bundle: SpendBundle,
    pub additions: Vec<Coin>,
    pub removals: Vec<Coin>,
    pub sent_to: Vec<TransactionPeer>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TransactionsInfo {
    pub aggregated_signature: String,
    pub generator_refs_root: String,
    pub generator_root: String,
    pub cost: u64,
    pub fees: u64,
    pub reward_claims_incorporated: Vec<Coin>,
}

#[derive(Deserialize)]
pub enum TXStatus {
    SUCCESS = 1,
    PENDING = 2,
    FAILED = 3,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UnfinishedBlock {
    pub challenge_chain_sp_proof: Option<VdfProof>,
    pub reward_chain_sp_proof: Option<VdfProof>,
    pub foliage: Foliage,
    pub foliage_transaction_block: Option<FoliageTransactionBlock>,
    pub transactions_filter: String,
    pub finished_sub_slots: Vec<SubSlotBundle>,
    pub reward_chain_block: RewardChainBlockUnfinished,
    pub transactions_info: Option<TransactionsInfo>,
    pub transactions_generator: Option<String>,
    pub transactions_generator_ref_list: Option<Vec<u32>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VdfInfo {
    pub challenge: String,
    pub output: VdfOutput,
    pub number_of_iterations: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VdfOutput {
    pub data: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VdfProof {
    pub normalized_to_identity: bool,
    pub witness: String,
    pub witness_type: u8,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WalletBalance {
    pub wallet_id: u32,
    pub pending_coin_removal_count: u32,
    pub unspent_coin_count: u32,
    pub confirmed_wallet_balance: u64,
    pub max_send_amount: u64,
    pub pending_change: u64,
    pub spendable_balance: u64,
    pub unconfirmed_wallet_balance: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WalletInfo {
    pub data: String,
    pub name: String,
    pub id: u32,
    #[serde(alias = "type")]
    pub wallet_type: WalletType,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WalletSync {
    pub genesis_initialized: bool,
    pub synced: bool,
    pub syncing: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum WalletType {
    StandardWallet = 0,
    RateLimited = 1,
    AtomicSwap = 2,
    AuthorizedPayee = 3,
    MultiSig = 4,
    Custody = 5,
    ColouredCoin = 6,
    RECOVERABLE = 7,
    DistributedId = 8,
    PoolingWallet = 9,
}
