#![cfg(test)]

use crate::Propagator;
use crate::propagators::ALL_DIFFERENT_INSTANCES;
use crate::propagators::ProofTestRunner;

#[test]
fn all_different_checker_test_0() {
    let runner = ProofTestRunner::new_runner(ALL_DIFFERENT_INSTANCES[0], Propagator::AllDifferent);
    let result = runner.run();

    if let Err(e) = result {
        panic!("Failed to check inference: {e:#?}");
    }
}

#[test]
fn all_different_checker_test_1() {
    let runner = ProofTestRunner::new_runner(ALL_DIFFERENT_INSTANCES[1], Propagator::AllDifferent);
    let result = runner.run();

    if let Err(e) = result {
        panic!("Failed to check inference: {e:#?}");
    }
}

#[test]
fn all_different_checker_test_2() {
    let runner = ProofTestRunner::new_runner(ALL_DIFFERENT_INSTANCES[2], Propagator::AllDifferent);
    let result = runner.run();

    if let Err(e) = result {
        panic!("Failed to check inference: {e:#?}");
    }
}

#[test]
fn all_different_checker_test_3() {
    let runner = ProofTestRunner::new_runner(ALL_DIFFERENT_INSTANCES[3], Propagator::AllDifferent);
    let result = runner.run();

    if let Err(e) = result {
        panic!("Failed to check inference: {e:#?}");
    }
}

#[test]
fn all_different_checker_test_0_invalid() {
    let runner = ProofTestRunner::new_runner(ALL_DIFFERENT_INSTANCES[0], Propagator::AllDifferent)
        .invalid_checks();
    let result = runner.run();

    if let Err(e) = result {
        panic!("Failed to check inference: {e:#?}");
    }
}

#[test]
fn all_different_checker_test_1_invalid() {
    let runner = ProofTestRunner::new_runner(ALL_DIFFERENT_INSTANCES[1], Propagator::AllDifferent)
        .invalid_checks();
    let result = runner.run();

    if let Err(e) = result {
        panic!("Failed to check inference: {e:#?}");
    }
}

#[test]
fn all_different_checker_test_2_invalid() {
    let runner = ProofTestRunner::new_runner(ALL_DIFFERENT_INSTANCES[2], Propagator::AllDifferent)
        .invalid_checks();
    let result = runner.run();

    if let Err(e) = result {
        panic!("Failed to check inference: {e:#?}");
    }
}

#[test]
fn all_different_checker_test_3_invalid() {
    let runner = ProofTestRunner::new_runner(ALL_DIFFERENT_INSTANCES[3], Propagator::AllDifferent)
        .invalid_checks();
    let result = runner.run();

    if let Err(e) = result {
        panic!("Failed to check inference: {e:#?}");
    }
}
