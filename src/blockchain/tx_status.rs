use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub enum TXStatus {
    SUCCESS = 1,
    PENDING = 2,
    FAILED = 3,
}
