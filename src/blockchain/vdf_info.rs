use crate::blockchain::vdf_output::VdfOutput;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct VdfInfo {
    pub challenge: String,
    pub output: VdfOutput,
    pub number_of_iterations: u64,
}
