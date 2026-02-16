use implementation::propagators::circuit::CircuitConstructor;
use implementation::propagators::circuit::CircuitExplanationType;
use pumpkin_core::constraints::Constraint;
use pumpkin_core::proof::ConstraintTag;
use pumpkin_core::variables::IntegerVariable;

/// Creates the [`Constraint`] that enforces that all the given `variables` are distinct.
pub fn circuit<Var: IntegerVariable + 'static>(
    variables: impl Into<Box<[Var]>>,
    constraint_tag: ConstraintTag,
    conflict_detection_only: bool,
    explanation_type: CircuitExplanationType,
) -> impl Constraint {
    let variables: Box<[Var]> = variables.into();

    CircuitConstructor {
        successors: variables,
        constraint_tag,
        conflict_detection_only,
        explanation_type,
    }
}
