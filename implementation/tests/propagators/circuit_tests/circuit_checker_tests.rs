#![cfg(test)]

use std::rc::Rc;

use drcp_format::IntAtomic;
use implementation::propagators::circuit::CircuitChecker;
use pumpkin_checking::InferenceChecker;
use pumpkin_checking::VariableState;

use crate::Propagator;
use crate::propagators::ProofTestRunner;
use crate::propagators::TSP_DIRECT_INSTANCES;
use crate::propagators::TSP_INSTANCES;
use crate::propagators::model::Atomic;
use crate::propagators::model::Variable;

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
        ProofTestRunner::new_runner(TSP_DIRECT_INSTANCES[0], Propagator::Circuit).invalid_checks();
    let result = runner.run();

    if let Err(e) = result {
        panic!("Failed to check inference: {e:#?}");
    }
}

#[test]
fn circuit_checker_test_1_invalid() {
    let runner =
        ProofTestRunner::new_runner(TSP_DIRECT_INSTANCES[1], Propagator::Circuit).invalid_checks();
    let result = runner.run();

    if let Err(e) = result {
        panic!("Failed to check inference: {e:#?}");
    }
}

#[test]
fn circuit_checker_test_2_invalid() {
    let runner =
        ProofTestRunner::new_runner(TSP_DIRECT_INSTANCES[2], Propagator::Circuit).invalid_checks();
    let result = runner.run();

    if let Err(e) = result {
        panic!("Failed to check inference: {e:#?}");
    }
}

#[test]
fn circuit_checker_test_3_invalid() {
    let runner =
        ProofTestRunner::new_runner(TSP_DIRECT_INSTANCES[3], Propagator::Circuit).invalid_checks();
    let result = runner.run();

    if let Err(e) = result {
        panic!("Failed to check inference: {e:#?}");
    }
}
use fzn_rs::VariableExpr;

#[test]
fn circuit_hamiltonian_path_checker() {
    let x: Variable = VariableExpr::Identifier(Rc::from("x")).into();
    let y: Variable = VariableExpr::Identifier(Rc::from("y")).into();
    let z: Variable = VariableExpr::Identifier(Rc::from("z")).into();

    let premises = vec![
        Atomic::IntAtomic(IntAtomic {
            name: "x".into(),
            comparison: drcp_format::IntComparison::Equal,
            value: 2,
        }),
        Atomic::IntAtomic(IntAtomic {
            name: "y".into(),
            comparison: drcp_format::IntComparison::Equal,
            value: 3,
        }),
        Atomic::IntAtomic(IntAtomic {
            name: "z".into(),
            comparison: drcp_format::IntComparison::Equal,
            value: 1,
        }),
    ];

    let variable_state = VariableState::prepare_for_conflict_check(premises.clone(), None)
        .expect("no mutually exclusive atomics");

    let checker = CircuitChecker {
        successors: vec![x, y, z],
    };

    assert!(
        !checker.check(variable_state, &premises, None),
        "If there is a cycle concerning all variables, then there is no conflict, so the checker should not pas it"
    )
}
