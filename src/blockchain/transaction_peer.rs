use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct TransactionPeer {
    pub peer: String,
    pub error: String,
    pub status: u32,
}
