//! Contains algorithms for conflict analysis, core extraction, and clause minimisation.
//! The algorithms use resolution and implement the 1uip and all decision literal learning schemes
mod conflict_analysis_context;
mod conflict_resolver;
mod learned_nogood;
mod nogood_minimiser;

pub use conflict_analysis_context::ConflictAnalysisContext;
pub use conflict_resolver::ConflictResolver;
pub(crate) use learned_nogood::LearnedNogood;
pub use nogood_minimiser::*;
