use pumpkin_checking::AtomicConstraint;

/// An inference used to support a deduction.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SupportingInference<Atomic> {
    /// The premises of the inference.
    pub premises: Vec<Atomic>,
    /// The consequent of the inference.
    ///
    /// [`None`] represents the literal false. I.e., if the consequent is [`None`], then the
    /// premises imply false.
    pub consequent: Option<Atomic>,
}

pub fn verify_deduction<Atomic>(
    _premises: impl IntoIterator<Item = Atomic>,
    _inferences: impl IntoIterator<Item = SupportingInference<Atomic>>,
) where
    Atomic: AtomicConstraint,
{
    todo!()
}
