#![cfg(test)]

use crate::LINEAR_INSTANCES;
use crate::ProofTestRunner;
use crate::Propagator;

#[test]
fn linear_checker_test_0() {
    let runner =
        ProofTestRunner::new_runner(LINEAR_INSTANCES[0], Propagator::Linear).check_deductions();
    let result = runner.run();

    if let Err(e) = result {
        panic!("Failed to check inference: {e:#?}");
    }
}

#[test]
fn linear_checker_test_1() {
    let runner =
        ProofTestRunner::new_runner(LINEAR_INSTANCES[1], Propagator::Linear).check_deductions();
    let result = runner.run();

    if let Err(e) = result {
        panic!("Failed to check inference: {e:#?}");
    }
}

#[test]
fn linear_checker_test_2() {
    let runner =
        ProofTestRunner::new_runner(LINEAR_INSTANCES[2], Propagator::Linear).check_deductions();
    let result = runner.run();

    if let Err(e) = result {
        panic!("Failed to check inference: {e:#?}");
    }
}

#[test]
fn linear_checker_test_3() {
    let runner =
        ProofTestRunner::new_runner(LINEAR_INSTANCES[3], Propagator::Linear).check_deductions();
    let result = runner.run();

    if let Err(e) = result {
        panic!("Failed to check inference: {e:#?}");
    }
}

#[test]
fn linear_checker_test_0_invalid() {
    let runner = ProofTestRunner::new_runner(LINEAR_INSTANCES[0], Propagator::Linear)
        .invalid_deductions_checks();
    let result = runner.run();

    if let Err(e) = result {
        panic!("Failed to check inference: {e:#?}");
    }
}

#[test]
fn linear_checker_test_1_invalid() {
    let runner = ProofTestRunner::new_runner(LINEAR_INSTANCES[1], Propagator::Linear)
        .invalid_deductions_checks();
    let result = runner.run();

    if let Err(e) = result {
        panic!("Failed to check inference: {e:#?}");
    }
}

#[test]
fn linear_checker_test_2_invalid() {
    let runner = ProofTestRunner::new_runner(LINEAR_INSTANCES[2], Propagator::Linear)
        .invalid_deductions_checks();
    let result = runner.run();

    if let Err(e) = result {
        panic!("Failed to check inference: {e:#?}");
    }
}

#[test]
fn linear_checker_test_3_invalid() {
    let runner = ProofTestRunner::new_runner(LINEAR_INSTANCES[3], Propagator::Linear)
        .invalid_deductions_checks();
    let result = runner.run();

    if let Err(e) = result {
        panic!("Failed to check inference: {e:#?}");
    }
}
