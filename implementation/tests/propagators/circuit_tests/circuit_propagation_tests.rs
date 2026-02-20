#![cfg(test)]
#![allow(
    deprecated,
    reason = "Will be refactored in the future using the state API"
)]

use implementation::propagators::circuit::CircuitConstructor;
use implementation::propagators::circuit::CircuitExplanationType;
use pumpkin_core::state::State;

use crate::Propagator;
use crate::propagators::ProofTestRunner;
use crate::propagators::TSP_DIRECT_INSTANCES;

#[test]
fn circuit_propagation_test_0() {
    let runner = ProofTestRunner::new_runner(TSP_DIRECT_INSTANCES[0], Propagator::Circuit)
        .check_propagations_only();
    let result = runner.run();

    if let Err(e) = result {
        panic!("Failed to check inference: {e:#?}");
    }
}

#[test]
fn circuit_propagation_test_1() {
    let runner = ProofTestRunner::new_runner(TSP_DIRECT_INSTANCES[1], Propagator::Circuit)
        .check_propagations_only();
    let result = runner.run();

    if let Err(e) = result {
        panic!("Failed to check inference: {e:#?}");
    }
}

#[test]
fn circuit_propagation_test_2() {
    let runner = ProofTestRunner::new_runner(TSP_DIRECT_INSTANCES[2], Propagator::Circuit)
        .check_propagations_only();
    let result = runner.run();

    if let Err(e) = result {
        panic!("Failed to check inference: {e:#?}");
    }
}

#[test]
fn circuit_propagation_test_3() {
    let runner = ProofTestRunner::new_runner(TSP_DIRECT_INSTANCES[3], Propagator::Circuit)
        .check_propagations_only();
    let result = runner.run();

    if let Err(e) = result {
        panic!("Failed to check inference: {e:#?}");
    }
}

#[test]
fn circuit_hamiltonian_path_propagation() {
    let mut state = State::default();

    let x = state.new_interval_variable(2, 2, None);
    let y = state.new_interval_variable(3, 3, None);
    let z = state.new_interval_variable(1, 1, None);

    let constraint_tag = state.new_constraint_tag();

    let _ = state.add_propagator(CircuitConstructor {
        successors: vec![x, y, z].into(),
        constraint_tag,
        conflict_detection_only: false,
        explanation_type: CircuitExplanationType::Direct,
    });

    let result = state.propagate_to_fixed_point();

    assert!(
        result.is_ok(),
        "If there is a cycle concerning all variables, then no conflict should be reported"
    )
}
