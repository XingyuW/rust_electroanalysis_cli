//! Independent MHI Phase-E validation facade.
//!
//! This module consumes frozen Phase-B/Phase-C artifacts and cannot invoke
//! their assessors.  Filesystem reads are kept in the runner/reader boundary;
//! statistics and classification helpers are deterministic and side-effect free.

pub mod approval;
pub mod error;
pub mod evaluation;
pub mod output;
pub mod protocol;
pub mod reader;
pub mod statistics;

pub use error::MhiValidationError;
pub use evaluation::evaluate_mhi_validation;
pub use protocol::MhiValidationProtocolV1;
pub use reader::ValidationInputs;
