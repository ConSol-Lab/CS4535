use pumpkin_core::conflict_resolving::ConflictAnalysisContext;
use pumpkin_core::conflict_resolving::ConflictResolver;
#[allow(unused, reason = "Will be used in the assignments")]
use pumpkin_core::propagation::ReadDomains;

/// [`ConflictResolver`] which resolves conflicts according to the all-decision learning approach.
///
/// This conflict resolver will derive a nogood that is implied by the constraints already present
/// in the solver. This new nogood is added as a constraint to the solver, and the solver
/// backtracks to the decision level at which the new constraint propagates.
#[allow(
    missing_copy_implementations,
    reason = "Might be uncopyable once implemented"
)]
#[derive(Clone, Debug)]
pub struct AllDecisionResolver;

impl AllDecisionResolver {
    #[allow(
        clippy::new_without_default,
        reason = "Might be non-default once implemented"
    )]
    pub fn new() -> Self {
        Self {}
    }
}

impl ConflictResolver for AllDecisionResolver {
    fn resolve_conflict(&mut self, _context: &mut ConflictAnalysisContext) {
        todo!()
    }
}
