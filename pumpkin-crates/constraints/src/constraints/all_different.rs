use implementation::propagators::all_different::AllDifferentConstructor;
use pumpkin_core::constraints::Constraint;
use pumpkin_core::proof::ConstraintTag;
use pumpkin_core::variables::IntegerVariable;

use crate::binary_not_equals;

/// Creates the [`Constraint`] that enforces that all the given `variables` are distinct.
pub fn all_different<Var: IntegerVariable + 'static>(
    variables: impl Into<Box<[Var]>>,
    constraint_tag: ConstraintTag,
    conflict_detection_only: bool,
    use_decomposition: bool,
) -> impl Constraint {
    AllDifferent {
        variables: variables.into(),
        constraint_tag,
        conflict_detection_only,
        use_decomposition,
    }
}

#[derive(Debug)]
pub struct AllDifferent<Var> {
    variables: Box<[Var]>,
    constraint_tag: ConstraintTag,
    conflict_detection_only: bool,
    use_decomposition: bool,
}

impl<Var: IntegerVariable + 'static> Constraint for AllDifferent<Var> {
    fn post(
        self,
        solver: &mut pumpkin_core::Solver,
    ) -> Result<(), pumpkin_core::ConstraintOperationError> {
        if self.use_decomposition {
            for i in 0..self.variables.len() {
                for j in i + 1..self.variables.len() {
                    binary_not_equals(
                        self.variables[i].clone(),
                        self.variables[j].clone(),
                        self.constraint_tag,
                        false,
                    )
                    .post(solver)?;
                }
            }
            Ok(())
        } else {
            AllDifferentConstructor {
                x: self.variables,
                constraint_tag: self.constraint_tag,
                conflict_detection_only: self.conflict_detection_only,
            }
            .post(solver)
        }
    }

    fn implied_by(
        self,
        solver: &mut pumpkin_core::Solver,
        reification_literal: pumpkin_core::variables::Literal,
    ) -> Result<(), pumpkin_core::ConstraintOperationError> {
        if self.use_decomposition {
            for i in 0..self.variables.len() {
                for j in i + 1..self.variables.len() {
                    binary_not_equals(
                        self.variables[i].clone(),
                        self.variables[j].clone(),
                        self.constraint_tag,
                        false,
                    )
                    .implied_by(solver, reification_literal)?;
                }
            }
            Ok(())
        } else {
            AllDifferentConstructor {
                x: self.variables,
                constraint_tag: self.constraint_tag,
                conflict_detection_only: self.conflict_detection_only,
            }
            .implied_by(solver, reification_literal)
        }
    }
}
