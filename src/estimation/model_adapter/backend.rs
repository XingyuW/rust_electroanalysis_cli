use crate::estimation_config::{CompiledEstimationProfile, EstimationModelBackend};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BackendSelection {
    pub backend: EstimationModelBackend,
    pub profile: Option<CompiledEstimationProfile>,
}

impl BackendSelection {
    pub fn legacy() -> Self {
        Self {
            backend: EstimationModelBackend::Legacy,
            profile: None,
        }
    }
}
