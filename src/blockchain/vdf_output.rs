use crate::blockchain::sized_bytes::UnsizedBytes;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VdfOutput {
    pub data: UnsizedBytes,
}
