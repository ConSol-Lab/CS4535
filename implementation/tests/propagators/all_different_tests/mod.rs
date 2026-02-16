use std::rc::Rc;

use implementation::propagators::all_different::AllDifferentConstructor;
use pumpkin_checking::AtomicConstraint;
use pumpkin_core::TestSolver;
use pumpkin_core::containers::HashMap;
use pumpkin_core::state::Conflict;
use pumpkin_core::state::PropagatorId;
use pumpkin_core::variables::DomainId;

use crate::CheckerError;
use crate::Propagator;
use crate::propagators::ProofTestRunner;
use crate::propagators::model::AllDifferent;
use crate::propagators::model::Fact;
use crate::propagators::model::Model;

mod all_different_checker_tests;
mod all_different_conflict_tests;
mod all_different_propagation_tests;

pub(crate) fn invalidate_all_different_fact(_all_different: &AllDifferent, _fact: &mut Fact) {
    // We create some random generator
    // let seed = all_different.variables.len() as u64
    //     + fact.premises.len() as u64
    //     + fact .premises .iter() .map(|premise| premise.value().unsigned_abs() as u64)
    //       .sum::<u64>();
    // let rng = SmallRng::seed_from_u64(seed);

    todo!()
}

pub(crate) fn recreate_conflict_all_different<'a>(
    instance: &'a str,
    all_different: &AllDifferent,
    fact: &Fact,
    model: &Model,
) -> Result<(), CheckerError<'a>> {
    assert!(fact.consequent.is_none());
    let (mut solver, variables) = ProofTestRunner::create_solver_for_fact(fact, model);

    let result = add_all_different_propagator(&mut solver, &variables, all_different, true);

    if result.is_err() {
        // We have been able to reproduce the conflict
        Ok(())
    } else {
        Err(CheckerError::ConflictCouldNotBeReproduced {
            fact: fact.clone(),
            instance,
            propagator: Propagator::AllDifferent,
            constraint: format!("{all_different:#?}"),
        })
    }
}

pub(crate) fn recreate_propagation_all_different<'a>(
    instance: &'a str,
    all_different: &AllDifferent,
    fact: &Fact,
    model: &Model,
) -> Result<(), CheckerError<'a>> {
    assert!(fact.consequent.is_some());
    let (mut solver, variables) = ProofTestRunner::create_solver_for_fact(fact, model);

    let result = add_all_different_propagator(&mut solver, &variables, all_different, false);

    let var = variables
        .get(&fact.consequent.as_ref().unwrap().identifier())
        .expect("Expected variable to exist");
    let consequent = ProofTestRunner::atomic_to_predicate(fact.consequent.as_ref().unwrap(), var);

    if result.is_ok() && solver.is_predicate_satisfied(consequent) {
        // We have been able to reproduce the conflict
        Ok(())
    } else {
        Err(CheckerError::PropagationCouldNotBeReproduced {
            fact: fact.clone(),
            instance,
            propagator: Propagator::AllDifferent,
            constraint: format!("{all_different:#?}"),
        })
    }
}

pub(crate) fn add_all_different_propagator(
    solver: &mut TestSolver,
    variables: &HashMap<Rc<str>, DomainId>,
    all_different: &AllDifferent,
    conflict_detection_only: bool,
) -> Result<PropagatorId, Conflict> {
    let constraint_tag = solver.new_constraint_tag();
    let variables = all_different
        .variables
        .iter()
        .filter(|variable| match &variable.0 {
            fzn_rs::VariableExpr::Identifier(ident) => variables.contains_key(ident),
            fzn_rs::VariableExpr::Constant(_) => true,
        })
        .map(|variable| match &variable.0 {
            fzn_rs::VariableExpr::Identifier(ident) => *variables.get(ident).unwrap(),
            fzn_rs::VariableExpr::Constant(constant) => solver.new_variable(*constant, *constant),
        })
        .collect::<Vec<_>>()
        .into();
    solver.new_propagator(AllDifferentConstructor {
        x: variables,
        constraint_tag,
        conflict_detection_only,
    })
}
