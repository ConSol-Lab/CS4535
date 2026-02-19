#![cfg(test)]
#![allow(
    deprecated,
    reason = "Will be refactored in the future using the state API"
)]

use crate::Propagator;
use crate::propagators::ProofTestRunner;
use crate::propagators::TSP_INSTANCES;

#[test]
fn circuit_propagation_test_0() {
    let runner = ProofTestRunner::new_runner(TSP_INSTANCES[0], Propagator::Circuit)
        .check_propagations_only();
    let result = runner.run();

    if let Err(e) = result {
        panic!("Failed to check inference: {e:#?}");
    }
}

#[test]
fn circuit_propagation_test_1() {
    let runner = ProofTestRunner::new_runner(TSP_INSTANCES[1], Propagator::Circuit)
        .check_propagations_only();
    let result = runner.run();

    if let Err(e) = result {
        panic!("Failed to check inference: {e:#?}");
    }
}

#[test]
fn circuit_propagation_test_2() {
    let runner = ProofTestRunner::new_runner(TSP_INSTANCES[2], Propagator::Circuit)
        .check_propagations_only();
    let result = runner.run();

    if let Err(e) = result {
        panic!("Failed to check inference: {e:#?}");
    }
}

#[test]
fn circuit_propagation_test_3() {
    let runner = ProofTestRunner::new_runner(TSP_INSTANCES[3], Propagator::Circuit)
        .check_propagations_only();
    let result = runner.run();

    if let Err(e) = result {
        panic!("Failed to check inference: {e:#?}");
    }
}
