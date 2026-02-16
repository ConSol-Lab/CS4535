mod checker;
mod propagator;

pub use checker::*;
pub use propagator::*;

/// The explanation used for explaining Circuit propagations and conflicts.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
pub enum CircuitExplanationType {
    // Explanation using only equality predicates.
    #[default]
    Direct,
    // Explanation using only disequality predicates.
    Indirect,
}
