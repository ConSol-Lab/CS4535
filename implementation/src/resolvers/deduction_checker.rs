use pumpkin_core::conflict_resolving::Atomic;
use pumpkin_core::conflict_resolving::DeductionChecker;
use pumpkin_core::conflict_resolving::SupportingInference;

#[derive(Debug, Copy, Clone)]
pub struct DeductionCheckerImpl;

impl DeductionChecker for DeductionCheckerImpl {
    fn verify_deduction(
        &self,
        _premises: Vec<Atomic>,
        _inferences: Vec<SupportingInference>,
    ) -> bool {
        todo!()
    }
}
