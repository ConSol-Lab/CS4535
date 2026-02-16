#![cfg(test)]
#![allow(
    deprecated,
    reason = "Will be refactored in the future using the state API"
)]

use crate::Propagator;
use crate::propagators::ALL_DIFFERENT_INSTANCES;
use crate::propagators::ProofTestRunner;

#[test]
fn all_different_propagation_test_0() {
    let runner = ProofTestRunner::new_runner(ALL_DIFFERENT_INSTANCES[0], Propagator::AllDifferent)
        .check_propagations_only();
    let result = runner.run();

    if let Err(e) = result {
        panic!("Failed to check inference: {e:#?}");
    }
}

#[test]
fn all_different_propagation_test_1() {
    let runner = ProofTestRunner::new_runner(ALL_DIFFERENT_INSTANCES[1], Propagator::AllDifferent)
        .check_propagations_only();
    let result = runner.run();

    if let Err(e) = result {
        panic!("Failed to check inference: {e:#?}");
    }
}

#[test]
fn all_different_propagation_test_2() {
    let runner = ProofTestRunner::new_runner(ALL_DIFFERENT_INSTANCES[2], Propagator::AllDifferent)
        .check_propagations_only();
    let result = runner.run();

    if let Err(e) = result {
        panic!("Failed to check inference: {e:#?}");
    }
}

#[test]
fn all_different_propagation_test_3() {
    let runner = ProofTestRunner::new_runner(ALL_DIFFERENT_INSTANCES[3], Propagator::AllDifferent)
        .check_propagations_only();
    let result = runner.run();

    if let Err(e) = result {
        panic!("Failed to check inference: {e:#?}");
    }
}
