use implementation::propagators::circuit::CircuitConstructor;
use pumpkin_core::constraints::Constraint;
use pumpkin_core::proof::ConstraintTag;
use pumpkin_core::variables::IntegerVariable;

use crate::constraints;

/// Creates the [`Constraint`] that enforces that all the given `variables` are distinct.
pub fn circuit<Var: IntegerVariable + 'static>(
    variables: impl Into<Box<[Var]>>,
    constraint_tag: ConstraintTag,
    conflict_detection_only: bool,
) -> impl Constraint {
    let variables: Box<[Var]> = variables.into();
    Circuit {
        variables,
        constraint_tag,
        conflict_detection_only,
    }
}

#[derive(Debug)]
pub struct Circuit<Var> {
    variables: Box<[Var]>,
    constraint_tag: ConstraintTag,
    conflict_detection_only: bool,
}

impl<Var: IntegerVariable + 'static> Constraint for Circuit<Var> {
    fn post(
        self,
        solver: &mut pumpkin_core::Solver,
    ) -> Result<(), pumpkin_core::ConstraintOperationError> {
        constraints::all_different(self.variables.clone(), self.constraint_tag, false, true)
            .post(solver)?;

        CircuitConstructor {
            successors: self.variables,
            constraint_tag: self.constraint_tag,
            conflict_detection_only: self.conflict_detection_only,
        }
        .post(solver)
    }

    fn implied_by(
        self,
        _solver: &mut pumpkin_core::Solver,
        _reification_literal: pumpkin_core::variables::Literal,
    ) -> Result<(), pumpkin_core::ConstraintOperationError> {
        unimplemented!()
    }
}
