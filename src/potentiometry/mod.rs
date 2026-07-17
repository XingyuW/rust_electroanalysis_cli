//! Mechanism-oriented, but mechanism-neutral, potentiometric analysis.
//!
//! This namespace owns time-domain response models and algorithms.  It does
//! not assign electrochemical mechanisms to fitted time constants.

pub mod error;
pub mod transient;

pub use error::PotentiometryError;
pub use transient::{TransientAnalysisOptions, analyze_experiment};
