//! Contains algorithms for conflict analysis, core extraction, and clause minimisation.
//! The algorithms use resolution and implement the 1uip and all decision literal learning schemes
mod atomic;
mod conflict_analysis_context;
mod conflict_resolver;
mod learned_nogood;
mod nogood_minimiser;

pub use atomic::*;
pub use conflict_analysis_context::ConflictAnalysisContext;
pub use conflict_resolver::ConflictResolver;
pub(crate) use learned_nogood::LearnedNogood;
pub use nogood_minimiser::*;

/// An inference used to support a deduction.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SupportingInference {
    /// The premises of the inference.
    pub premises: Vec<Atomic>,
    /// The consequent of the inference.
    ///
    /// [`None`] represents the literal false. I.e., if the consequent is [`None`], then the
    /// premises imply false.
    pub consequent: Option<Atomic>,
}

pub trait DeductionChecker {
    fn verify_deduction<Atomic>(
        _premises: impl IntoIterator<Item = Atomic>,
        _inferences: impl IntoIterator<Item = SupportingInference>,
    ) -> bool {
        todo!()
    }
}
