use pumpkin_core::conflict_resolving::ConflictAnalysisContext;
use pumpkin_core::conflict_resolving::ConflictResolver;
#[allow(unused, reason = "Will be used in the assignments")]
use pumpkin_core::propagation::ReadDomains;

/// [`ConflictResolver`] which resolves conflicts according to the CDCL procedure.
///
/// This conflict resolver will derive a nogood that is implied by the constraints already present
/// in the solver. This new nogood is added as a constraint to the solver, and the solver
/// backtracks to the decision level at which the new constraint propagates.
///
/// For an in-depth explanation and overview of CDCL and UIP, see \[1\].
///
/// # Bibliography
/// \[1\] J. Marques-Silva, I. Lynce, and S. Malik, ‘Conflict-driven clause learning SAT solvers’,
/// Handbook of satisfiability, pp. 131–153, 2009.
#[allow(
    missing_copy_implementations,
    reason = "Might be uncopyable once implemented"
)]
#[derive(Clone, Debug)]
pub struct OneUIP {}

impl OneUIP {
    #[allow(
        clippy::new_without_default,
        reason = "Might be non-default once implemented"
    )]
    pub fn new() -> Self {
        Self {}
    }
}

impl ConflictResolver for OneUIP {
    fn resolve_conflict(&mut self, _context: &mut ConflictAnalysisContext) {
        todo!()
    }
}
