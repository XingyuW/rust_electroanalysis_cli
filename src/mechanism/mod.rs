//! Integration layer for EIS/transient timescale comparison.

pub mod error;
pub mod evidence;
pub mod interpretation;
pub mod matching;
pub mod timescale;
pub mod trend;
pub mod uncertainty;

pub use evidence::*;
pub use matching::*;
pub use timescale::*;
pub use trend::*;
