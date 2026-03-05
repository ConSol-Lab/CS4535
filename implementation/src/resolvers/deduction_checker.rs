use pumpkin_core::conflict_resolving::DeductionChecker;
use pumpkin_core::conflict_resolving::SupportingInference;

#[derive(Debug, Copy, Clone)]
pub struct DeductionCheckerImpl;

impl DeductionChecker for DeductionCheckerImpl {
    fn verify_deduction<Atomic>(
        _premises: impl IntoIterator<Item = Atomic>,
        _inferences: impl IntoIterator<Item = SupportingInference>,
    ) -> bool {
        todo!()
    }
}
