use crate::blockchain::sized_bytes::Bytes32;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Constants {
    pub genesis_challenge: Bytes32,
    pub min_plot_size: u8,
    pub max_plot_size: u8,
    pub max_transaction_amount: u64,
    pub difficulty_constant_factor: u128,
    pub pool_sub_slot_iters: u64,
    pub iters_limit: u64,
    pub protocol_version: i64,
}

lazy_static! {
    pub static ref MAINNET: Constants = Constants {
        genesis_challenge: "ccd5bb71183532bff220ba46c268991a3ff07eb358e8255a65c30a2dce0e5fbb"
            .into(),
        min_plot_size: 32,
        max_plot_size: 50,
        max_transaction_amount: 446250000000000,
        difficulty_constant_factor: 2 ^ 67,
        pool_sub_slot_iters: 37600000000,
        iters_limit: 37600000000 / 64,
        protocol_version: 1,
    };
    pub static ref TESTNET10: Constants = Constants {
        genesis_challenge: "ae83525ba8d1dd3f09b277de18ca3e43fc0af20d20c4b3e92ef2a48bd291ccb2"
            .into(),
        min_plot_size: 25,
        max_plot_size: 50,
        max_transaction_amount: 446250000000000,
        difficulty_constant_factor: 10052721566054,
        pool_sub_slot_iters: 37600000000,
        iters_limit: 37600000000 / 64,
        protocol_version: 1,
    };
}
