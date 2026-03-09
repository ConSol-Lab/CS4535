use pumpkin_core::conflict_resolving::ConflictAnalysisContext;
use pumpkin_core::conflict_resolving::NogoodMinimiser;
use pumpkin_core::predicates::Predicate;

#[allow(
    missing_copy_implementations,
    reason = "Might not be copy after implementing"
)]
#[derive(Debug)]
pub struct SemanticMinimiser {}

impl SemanticMinimiser {
    #[allow(clippy::new_without_default, reason = "Might not be possible")]
    pub fn new() -> Self {
        Self {}
    }
}

impl NogoodMinimiser for SemanticMinimiser {
    fn minimise(&mut self, _context: &mut ConflictAnalysisContext, _nogood: &mut Vec<Predicate>) {
        todo!()
    }
}
