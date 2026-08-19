//! Phase-D reporting, plotting, and public scientific-output façade.

pub(crate) mod claims;
pub(crate) mod document;
pub mod error;
pub(crate) mod figures;
pub(crate) mod lineage;
pub(crate) mod projection;
pub(crate) mod reader;
pub(crate) mod tables;

pub use error::{AvailabilityReason, CompatibilityAxis, PublicReportError, PublicationPhase};
