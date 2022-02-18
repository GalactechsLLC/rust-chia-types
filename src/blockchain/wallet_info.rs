use crate::blockchain::wallet_type::WalletType;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct WalletInfo {
    pub data: String,
    pub name: String,
    pub id: u32,
    #[serde(alias = "type")]
    pub wallet_type: WalletType,
}
