#![cfg(test)]

use crate::Propagator;
use crate::propagators::ProofTestRunner;
use crate::propagators::TSP_INSTANCES;

#[test]
fn circuit_checker_test_0() {
    let runner = ProofTestRunner::new_runner(TSP_INSTANCES[0], Propagator::Circuit);
    let result = runner.run();

    if let Err(e) = result {
        panic!("Failed to check inference: {e:#?}");
    }
}

#[test]
fn circuit_checker_test_1() {
    let runner = ProofTestRunner::new_runner(TSP_INSTANCES[1], Propagator::Circuit);
    let result = runner.run();

    if let Err(e) = result {
        panic!("Failed to check inference: {e:#?}");
    }
}

#[test]
fn circuit_checker_test_2() {
    let runner = ProofTestRunner::new_runner(TSP_INSTANCES[2], Propagator::Circuit);
    let result = runner.run();

    if let Err(e) = result {
        panic!("Failed to check inference: {e:#?}");
    }
}

#[test]
fn circuit_checker_test_3() {
    let runner = ProofTestRunner::new_runner(TSP_INSTANCES[3], Propagator::Circuit);
    let result = runner.run();

    if let Err(e) = result {
        panic!("Failed to check inference: {e:#?}");
    }
}

#[test]
fn circuit_checker_test_0_invalid() {
    let runner =
        ProofTestRunner::new_runner(TSP_INSTANCES[0], Propagator::Circuit).invalid_checks();
    let result = runner.run();

    if let Err(e) = result {
        panic!("Failed to check inference: {e:#?}");
    }
}

#[test]
fn circuit_checker_test_1_invalid() {
    let runner =
        ProofTestRunner::new_runner(TSP_INSTANCES[1], Propagator::Circuit).invalid_checks();
    let result = runner.run();

    if let Err(e) = result {
        panic!("Failed to check inference: {e:#?}");
    }
}

#[test]
fn circuit_checker_test_2_invalid() {
    let runner =
        ProofTestRunner::new_runner(TSP_INSTANCES[2], Propagator::Circuit).invalid_checks();
    let result = runner.run();

    if let Err(e) = result {
        panic!("Failed to check inference: {e:#?}");
    }
}

#[test]
fn circuit_checker_test_3_invalid() {
    let runner =
        ProofTestRunner::new_runner(TSP_INSTANCES[3], Propagator::Circuit).invalid_checks();
    let result = runner.run();

    if let Err(e) = result {
        panic!("Failed to check inference: {e:#?}");
    }
}
