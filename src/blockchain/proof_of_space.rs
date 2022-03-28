use crate::blockchain::sized_bytes::{Bytes32, Bytes48, SizedBytes};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::error::Error;

pub const NUMBER_ZERO_BITS_PLOT_FILTER: i32 = 9;

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct ProofOfSpace {
    pub challenge: Bytes32,
    pub pool_contract_puzzle_hash: Option<Bytes32>,
    pub plot_public_key: Bytes48,
    pub pool_public_key: Option<Bytes48>,
    pub proof: Vec<u8>,
    pub size: u8,
}
impl ProofOfSpace {
    fn get_plot_id(&self) -> Result<Option<Bytes32>, Box<dyn Error>> {
        if self.pool_public_key.is_none() || self.pool_contract_puzzle_hash.is_none() {
            if self.pool_public_key.is_none() && self.pool_contract_puzzle_hash.is_some() {
                return Ok(Some(self.calculate_plot_id_puzzle_hash(
                    &self.pool_contract_puzzle_hash.clone().unwrap(),
                    &self.plot_public_key,
                )?));
            }
            return Ok(Some(self.calculate_plot_id_public_key(
                &self.pool_public_key.clone().unwrap(),
                &self.plot_public_key,
            )?));
        } else {
            return Ok(None);
        }
    }

    pub fn verify_and_get_quality_string(
        &self,
        original_challenge_hash: &Bytes32,
        signage_point: &Bytes32,
        min_size: u8,
        max_size: u8,
    ) -> Result<Option<Bytes32>, Box<dyn Error>> {
        if self.pool_public_key.is_none() && self.pool_contract_puzzle_hash.is_none() {
            //Logger.getInstance().log(Level.WARNING, "Failed to Verify ProofOfSpace: null value for pool_public_key and pool_contract_puzzle_hash");
            return Ok(None);
        }
        if self.pool_public_key.is_some() && self.pool_contract_puzzle_hash.is_some() {
            //Logger.getInstance().log(Level.WARNING, "Failed to Verify ProofOfSpace: Non Null value for both for pool_public_key and pool_contract_puzzle_hash");
            return Ok(None);
        }
        if self.size < min_size {
            //Logger.getInstance().log(Level.WARNING, "Failed to Verify ProofOfSpace: Plot failed MIN_PLOT_SIZE");
            return Ok(None);
        }
        if self.size > max_size {
            //Logger.getInstance().log(Level.WARNING, "Failed to Verify ProofOfSpace: Plot failed MAX_PLOT_SIZE");
            return Ok(None);
        }
        let plot_id_result = self.get_plot_id()?;
        if plot_id_result == None {
            return Ok(None);
        }
        let plot_id = plot_id_result.unwrap();
        if &self.challenge
            != &self.calculate_pos_challenge(&plot_id, original_challenge_hash, signage_point)?
        {
            //Logger.getInstance().log(Level.WARNING, "Failed to Verify ProofOfSpace: New challenge is not challenge");
            return Ok(None);
        }
        if !&self.passes_plot_filter(&plot_id, original_challenge_hash, signage_point)? {
            //Logger.getInstance().log(Level.WARNING, "Failed to Verify ProofOfSpace: Plot Failed to Pass Filter");
            return Ok(None);
        }
        return Ok(Some(self.get_quality_string(&plot_id)?));
    }

    pub fn get_quality_string(&self, plot_id: &Bytes32) -> Result<Bytes32, Box<dyn Error>> {
        //Puzzles.validateProof(plot_id, this.size, this.challenge, this.proof)
        Ok(Bytes32::new(Vec::new()))
    }

    pub fn calculate_plot_id_public_key(
        &self,
        pool_public_key: &Bytes48,
        plot_public_key: &Bytes48,
    ) -> Result<Bytes32, Box<dyn Error>> {
        let mut to_hash: Vec<u8> = Vec::new();
        to_hash.extend(pool_public_key.to_bytes());
        to_hash.extend(plot_public_key.to_bytes());
        let mut hasher: Sha256 = Sha256::new();
        hasher.update(to_hash);
        Ok(Bytes32::new(hasher.finalize().to_vec()))
    }

    pub fn calculate_plot_id_puzzle_hash(
        &self,
        pool_contract_puzzle_hash: &Bytes32,
        plot_public_key: &Bytes48,
    ) -> Result<Bytes32, Box<dyn Error>> {
        let mut to_hash: Vec<u8> = Vec::new();
        to_hash.extend(pool_contract_puzzle_hash.to_bytes());
        to_hash.extend(plot_public_key.to_bytes());
        let mut hasher: Sha256 = Sha256::new();
        hasher.update(to_hash);
        Ok(Bytes32::new(hasher.finalize().to_vec()))
    }

    pub fn passes_plot_filter(
        &self,
        plot_id: &Bytes32,
        challenge_hash: &Bytes32,
        signage_point: &Bytes32,
    ) -> Result<bool, Box<dyn Error>> {
        let mut filter = [false; 256];
        let mut index = 0;
        for b in &self
            .calculate_plot_filter_input(plot_id, challenge_hash, signage_point)?
            .to_bytes()
        {
            for i in 7..=0 {
                filter[index] = (b >> i & 1) == 1;
                index += 1;
            }
        }
        for i in 0..NUMBER_ZERO_BITS_PLOT_FILTER {
            if filter[i as usize] {
                return Ok(false);
            }
        }
        return Ok(true);
    }

    pub fn calculate_plot_filter_input(
        &self,
        plot_id: &Bytes32,
        challenge_hash: &Bytes32,
        signage_point: &Bytes32,
    ) -> Result<Bytes32, Box<dyn Error>> {
        let mut to_hash: Vec<u8> = Vec::new();
        to_hash.extend(plot_id.to_bytes());
        to_hash.extend(challenge_hash.to_bytes());
        to_hash.extend(signage_point.to_bytes());
        let mut hasher: Sha256 = Sha256::new();
        hasher.update(to_hash);
        Ok(Bytes32::new(hasher.finalize().to_vec()))
    }

    pub fn calculate_pos_challenge(
        &self,
        plot_id: &Bytes32,
        challenge_hash: &Bytes32,
        signage_point: &Bytes32,
    ) -> Result<Bytes32, Box<dyn Error>> {
        let mut hasher: Sha256 = Sha256::new();
        let to_hash = &self.calculate_plot_filter_input(plot_id, challenge_hash, signage_point)?;
        hasher.update(to_hash.to_bytes());
        Ok(Bytes32::new(hasher.finalize().to_vec()))
    }

    pub fn hash(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut to_hash: Vec<u8> = Vec::new();
        to_hash.extend(&self.challenge.clone().to_bytes());
        match &self.pool_public_key {
            Some(public_key) => {
                to_hash.push(1);
                to_hash.extend(public_key.to_bytes());
            }
            None => {
                to_hash.push(0);
            }
        }
        match &self.pool_contract_puzzle_hash {
            Some(contract_hash) => {
                to_hash.push(1);
                to_hash.extend(contract_hash.to_bytes());
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
