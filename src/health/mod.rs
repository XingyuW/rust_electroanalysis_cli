//! Feature integration and transparent sensor-health assessment.
pub mod assessment;
pub mod baseline;
pub mod error;
pub mod evidence;
pub mod features;
pub mod normalization;
pub mod rules;
pub mod trend;

pub use crate::results::{
    FeatureComparability, HealthConfidence, HealthDomain, OverallHealthStatus,
};
