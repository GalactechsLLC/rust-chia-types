use crate::blockchain::sized_bytes::{Bytes32, Bytes48, SizedBytes};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::error::Error;

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct ProofOfSpace {
    pub challenge: Bytes32,
    pub pool_contract_puzzle_hash: Option<Bytes32>,
    pub plot_public_key: Bytes48,
    pub pool_public_key: Option<Bytes48>,
    pub proof: Vec<u8>,
    pub size: u8,
}
impl ProofOfSpace {
    pub fn hash(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut to_hash: Vec<u8> = Vec::new();
        to_hash.extend(&self.challenge.to_bytes());
        match &self.pool_public_key {
            Some(public_key) => {
                to_hash.push(1);
                to_hash.extend(&public_key.to_bytes());
            }
            None => {
                to_hash.push(0);
            }
        }
        match &self.pool_contract_puzzle_hash {
            Some(contract_hash) => {
                to_hash.push(1);
                to_hash.extend(&contract_hash.to_bytes());
            }
            None => {
                to_hash.push(0);
            }
        }
        to_hash.extend(&self.plot_public_key.to_bytes());
        to_hash.push(self.size);
        to_hash.extend(&self.proof);
        let mut hasher: Sha256 = Sha256::new();
        hasher.update(to_hash);
        Ok(hasher.finalize().to_vec())
    }
}
