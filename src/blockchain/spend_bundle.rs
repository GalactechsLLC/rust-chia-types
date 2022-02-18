use crate::blockchain::coin_spend::CoinSpend;
use crate::blockchain::sized_bytes::Bytes96;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct SpendBundle {
    pub coin_spends: Vec<CoinSpend>,
    pub aggregated_signature: Bytes96,
}
