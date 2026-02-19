mod circuit_checker_tests;
mod circuit_conflict_tests;
mod circuit_propagation_tests;

use std::rc::Rc;

use implementation::propagators::all_different::AllDifferentConstructor;
use implementation::propagators::circuit::CircuitConstructor;
use implementation::propagators::circuit::CircuitExplanationType;
use pumpkin_checking::AtomicConstraint;
use pumpkin_core::TestSolver;
use pumpkin_core::containers::HashMap;
use pumpkin_core::state::Conflict;
use pumpkin_core::state::PropagatorId;
use pumpkin_core::variables::DomainId;

use crate::CheckerError;
use crate::Propagator;
use crate::propagators::ProofTestRunner;
use crate::propagators::model::Circuit;
use crate::propagators::model::Fact;
use crate::propagators::model::Model;

pub(crate) fn invalidate_circuit_fact(_circuit: &Circuit, _fact: &mut Fact) {
    // We create some random generator
    // let seed = all_different.variables.len() as u64
    //     + fact.premises.len() as u64
    //     + fact .premises .iter() .map(|premise| premise.value().unsigned_abs() as u64)
    //       .sum::<u64>();
    // let rng = SmallRng::seed_from_u64(seed);

    todo!()
}

pub(crate) fn recreate_conflict_circuit<'a>(
    instance: &'a str,
    circuit: &Circuit,
    fact: &Fact,
    model: &Model,
) -> Result<(), CheckerError<'a>> {
    assert!(fact.consequent.is_none());
    let (mut solver, variables) = ProofTestRunner::create_solver_for_fact(fact, model);

    let result = add_circuit_propagator(model, &mut solver, &variables, circuit, true);

    if result.is_err() {
        // We have been able to reproduce the conflict
        Ok(())
    } else {
        Err(CheckerError::ConflictCouldNotBeReproduced {
            fact: fact.clone(),
            instance,
            propagator: Propagator::Circuit,
            constraint: format!("{circuit:#?}"),
        })
    }
}

pub(crate) fn recreate_propagation_circuit<'a>(
    instance: &'a str,
    circuit: &Circuit,
    fact: &Fact,
    model: &Model,
) -> Result<(), CheckerError<'a>> {
    assert!(fact.consequent.is_some());
    let (mut solver, variables) = ProofTestRunner::create_solver_for_fact(fact, model);

    let result = add_circuit_propagator(model, &mut solver, &variables, circuit, false);

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
            propagator: Propagator::Circuit,
            constraint: format!("{circuit:#?}"),
        })
    }
}

pub(crate) fn add_circuit_propagator(
    model: &Model,
    solver: &mut TestSolver,
    variables: &HashMap<Rc<str>, DomainId>,
    circuit: &Circuit,
    conflict_detection_only: bool,
) -> Result<PropagatorId, Conflict> {
    let constraint_tag = solver.new_constraint_tag();
    let variables = circuit
        .successors
        .iter()
        .map(|variable| match &variable.0 {
            fzn_rs::VariableExpr::Identifier(ident) => {
                if let Some(var) = variables.get(ident) {
                    var.clone()
                } else {
                    // We add all variables to the model; not that interesting if you only get a
                    // cycle
                    let domain = model.get_domain(&ident);

                    let var = match domain {
                        fzn_rs::ast::Domain::UnboundedInt => unimplemented!(),
                        fzn_rs::ast::Domain::Int(range_list) => solver.new_variable(
                            (*range_list.lower_bound()) as i32,
                            (*range_list.upper_bound()) as i32,
                        ),
                        fzn_rs::ast::Domain::Bool => solver.new_variable(0, 1),
                    };

                    var
                }
            }
            fzn_rs::VariableExpr::Constant(constant) => solver.new_variable(*constant, *constant),
        })
        .collect::<Vec<_>>()
        .into();

    solver.new_propagator(CircuitConstructor {
        successors: variables,
        constraint_tag,
        conflict_detection_only,
        explanation_type: CircuitExplanationType::default(),
    })
}
