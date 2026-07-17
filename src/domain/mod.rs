//! Shared application-domain contracts.
//!
//! Phase 0 keeps the existing scientific data structures in `data_file` and
//! `impedance`.  This module provides the cross-cutting typed errors that
//! connect those subsystems to configuration and command orchestration.

pub mod errors;

pub use errors::{
    ConfigurationError, DataParsingError, FittingError, PlottingError, ReportingError,
    WorkspaceError,
};
