//! Integration layer for EIS/transient timescale comparison.

pub mod amplitude;
pub mod config;
pub mod error;
pub mod evaluation;
pub mod evidence;
pub mod history;
pub mod identifiability;
pub mod interpretation;
pub mod matching;
pub mod model_mapping;
pub mod preparation;
pub mod promotion;
pub mod repeatability;
pub mod temporal;
pub mod timescale;
pub mod trend;
pub mod uncertainty;
pub mod validation;

pub use config::*;
pub use evidence::*;
pub use matching::*;
pub use model_mapping::*;
pub use timescale::*;
pub use trend::*;
