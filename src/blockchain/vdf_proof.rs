use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct VdfProof {
    pub normalized_to_identity: bool,
    pub witness: String,
    pub witness_type: u8,
}
