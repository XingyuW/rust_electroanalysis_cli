//! Estimation-owned boundary around compiled ISM models.
//!
//! This module deliberately depends on both estimation and `model`; the model
//! core remains unaware of filters, calibration artifacts, and covariance
//! policy.
pub mod backend;
pub mod covariance_binding;
pub mod input_binding;
pub mod output_binding;
pub mod profile;
pub mod state_binding;

pub use backend::BackendSelection;
pub use state_binding::StateBinding;
