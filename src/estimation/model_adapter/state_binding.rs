use serde::{Deserialize, Serialize};

/// Stable mapping recorded by compiled-estimation artifacts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StateBinding {
    pub state_id: String,
    pub estimator_index: usize,
    pub ownership: String,
}
