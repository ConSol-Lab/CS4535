#![allow(
    deprecated,
    reason = "Will be refactored in the future using the state API"
)]
mod model;
use drcp_format::Deduction;
use implementation::conflict_analysis::DeductionCheckerImpl;
use implementation::propagators::cumulative::Task;
use pumpkin_checking::AtomicConstraint;
use pumpkin_checking::InferenceChecker;
use pumpkin_checking::VariableState;
use pumpkin_core::TestSolver;
use pumpkin_core::conflict_resolving::DeductionChecker;
use pumpkin_core::conflict_resolving::SupportingInference;
use pumpkin_core::containers::HashMap;
use pumpkin_core::options::ConflictResolverType;
use pumpkin_core::predicate;
use pumpkin_core::predicates::Predicate;
use pumpkin_core::variables::DomainId;

mod all_different_tests;
mod circuit_tests;
mod cumulative_tests;
mod linear_tests;

use std::collections::BTreeMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::rc::Rc;

pub use cumulative_tests::set_up_cumulative_state;
use drcp_format::reader::ProofReader;
use implementation::propagators::all_different::AllDifferentChecker;
use implementation::propagators::circuit::CircuitChecker;
use implementation::propagators::cumulative::CumulativeChecker;
use implementation::propagators::linear::LinearChecker;
pub use linear_tests::set_up_linear_leq_state;
use pumpkin_core::conflict_resolving::Atomic;

use crate::propagators::all_different_tests::invalidate_all_different_fact;
use crate::propagators::all_different_tests::recreate_conflict_all_different;
use crate::propagators::all_different_tests::recreate_propagation_all_different;
use crate::propagators::circuit_tests::invalidate_circuit_fact;
use crate::propagators::circuit_tests::recreate_conflict_circuit;
use crate::propagators::circuit_tests::recreate_propagation_circuit;
use crate::propagators::cumulative_tests::invalidate_cumulative_fact;
use crate::propagators::cumulative_tests::recreate_conflict_cumulative;
use crate::propagators::cumulative_tests::recreate_propagation_cumulative;
use crate::propagators::linear_tests::invalidate_linear_fact;
use crate::propagators::linear_tests::recreate_conflict_linear;
use crate::propagators::linear_tests::recreate_propagation_linear;
pub(crate) use crate::propagators::model::Constraint;
use crate::propagators::model::Fact;
use crate::propagators::model::Linear;
pub(crate) use crate::propagators::model::Model;
use crate::propagators::model::Term;
use crate::propagators::model::parse_model;
use crate::resolvers::create_solver_with_constraints;
use crate::resolvers::invalidate_nogood_deduction;
use crate::resolvers::recreate_deduction;

pub(crate) struct ProofTestRunner<'a> {
    instance: &'a str,
    propagator: Propagator,

    run_checker: bool,
    check_invalid: bool,

    check_conflicts: bool,
    check_propagations: bool,

    check_learned_nogoods: bool,
    check_deductions: bool,

    resolver: ConflictResolverType,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum CheckerError<'a> {
    #[error(
        "The {propagator:?} checker was not able to check the inference {fact:#?} for instance {instance}\nConstraint: {constraint:#?}"
    )]
    CouldNotCheck {
        fact: Fact,
        instance: &'a str,
        propagator: Propagator,
        constraint: Constraint,
    },
    #[error(
        "The {propagator:?} checker did not reject the inference {fact:#?} for instance {instance}\nConstraint: {constraint:#?}"
    )]
    CheckerDidNotReject {
        fact: Fact,
        instance: &'a str,
        propagator: Propagator,
        constraint: Constraint,
    },
    #[error(
        "The {propagator:?} propagator was not able to recreate the conflict described by {fact:#?} for instance {instance}\nConstraint: {constraint:#?}"
    )]
    ConflictCouldNotBeReproduced {
        fact: Fact,
        instance: &'a str,
        propagator: Propagator,
        constraint: Constraint,
    },
    #[error(
        "The {propagator:?} propagator was not able to recreate the propagation described by {fact:#?} for instance {instance}\nConstraint: {constraint:#?}"
    )]
    PropagationCouldNotBeReproduced {
        fact: Fact,
        instance: &'a str,
        propagator: Propagator,
        constraint: Constraint,
    },
    #[error(
        "The deduction checker was not able to check the deduction {deduction:#?} for instance {instance}"
    )]
    CouldNotCheckDeduction {
        deduction: Deduction<Rc<str>, i32>,
        instance: &'a str,
    },
    #[error(
        "The deduction checker did not reject deduction {deduction:#?} for instance {instance}"
    )]
    DeductionCheckerDidNotReject {
        deduction: Deduction<Rc<str>, i32>,
        instance: &'a str,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Propagator {
    Linear,
    Cumulative,
    Circuit,
    AllDifferent,
}

pub(crate) const LINEAR_INSTANCES: [&str; 4] = [
    "market_split_u3_01",
    "market_split_u3_02",
    "market_split_u3_03",
    "market_split_u3_04",
];

pub(crate) const RCPSP_INSTANCES: [&str; 4] = ["rcpsp00", "rcpsp01", "rcpsp02", "rcpsp03"];

pub(crate) const ALL_DIFFERENT_INSTANCES: [&str; 4] =
    ["sudoku_p0", "sudoku_p1", "sudoku_p3", "sudoku_p17"];

pub(crate) const TSP_INSTANCES: [&str; 4] = ["TSP_N5_3", "TSP_N10_0", "TSP_N10_1", "TSP_N10_2"];

pub(crate) const TSP_DIRECT_INSTANCES: [&str; 4] = [
    "TSP_N5_3_direct",
    "TSP_N10_0_direct",
    "TSP_N10_1_direct",
    "TSP_N10_2_direct",
];

impl<'a> ProofTestRunner<'a> {
    pub(crate) fn new_runner(instance: &'a str, propagator: Propagator) -> Self {
        Self {
            instance,
            propagator,

            run_checker: true,
            check_invalid: false,
            check_conflicts: false,
            check_propagations: false,

            check_learned_nogoods: false,
            check_deductions: false,

            resolver: ConflictResolverType::NoLearning,
        }
    }

    pub(crate) fn invalid_checks(mut self) -> Self {
        self.run_checker = true;
        self.check_invalid = true;

        self.check_conflicts = false;
        self.check_propagations = false;

        self.check_learned_nogoods = false;
        self.check_deductions = false;

        self
    }

    pub(crate) fn check_conflicts_only(mut self) -> Self {
        self.run_checker = false;
        self.check_invalid = false;

        self.check_conflicts = true;
        self.check_propagations = false;

        self.check_learned_nogoods = false;
        self.check_deductions = false;

        self
    }

    pub(crate) fn check_propagations_only(mut self) -> Self {
        self.run_checker = false;
        self.check_invalid = false;

        self.check_conflicts = false;
        self.check_propagations = true;

        self.check_learned_nogoods = false;
        self.check_deductions = false;

        self
    }

    pub(crate) fn check_deductions(mut self) -> Self {
        self.run_checker = false;
        self.check_invalid = false;

        self.check_conflicts = false;
        self.check_propagations = false;

        self.check_learned_nogoods = false;
        self.check_deductions = true;

        self
    }

    pub(crate) fn invalid_deductions_checks(mut self) -> Self {
        self.run_checker = false;
        self.check_invalid = true;

        self.check_conflicts = false;
        self.check_propagations = false;

        self.check_learned_nogoods = false;
        self.check_deductions = true;

        self
    }

    pub(crate) fn check_nogoods(mut self, resolver: ConflictResolverType) -> Self {
        self.run_checker = false;
        self.check_invalid = false;

        self.check_conflicts = false;
        self.check_propagations = false;

        self.check_learned_nogoods = true;
        self.check_deductions = false;

        self.resolver = resolver;

        self
    }

    pub(crate) fn run(&self) -> Result<(), CheckerError<'_>> {
        let model_path = Path::new("../pumpkin-checker/tests/valid_proofs")
            .join(format!("{}.fzn", self.instance));
        let model = parse_model(model_path.clone())
            .unwrap_or_else(|err| panic!("Failed to read model {model_path:?}\n{err:?}"));

        let proof_path = Path::new("../pumpkin-checker/tests/valid_proofs")
            .join(format!("{}.drcp", self.instance));
        let mut reader: ProofReader<BufReader<File>, i32> = ProofReader::new(BufReader::new(
            File::open(proof_path.clone())
                .unwrap_or_else(|_| panic!("Expected instance {proof_path:?} to exist")),
        ));

        let mut fact_database = BTreeMap::new();

        let mut solver_and_variables = if self.check_deductions || self.check_learned_nogoods {
            Some(create_solver_with_constraints(&model, self.resolver))
        } else {
            None
        };

        loop {
            let step = reader.next_step().expect("proofs are readable and valid");

            let Some(step) = step else {
                break;
            };

            match step {
                drcp_format::Step::Inference(inference) => {
                    let label = inference.label.expect("all inferences have labels");

                    {
                        let fact = Fact {
                            premises: inference.premises.iter().cloned().map(Into::into).collect(),
                            consequent: inference.consequent.clone().map(Into::into),
                        };
                        let _ = fact_database.insert(inference.constraint_id, fact);
                    }

                    // We do not check initial domains
                    if label.as_ref() == "initial_domain" || label.as_ref() == "nogood" {
                        continue;
                    }

                    let generated_by_constraint_id =
                        inference.generated_by.expect("all inferences have hints");

                    let generated_by = model
                        .get_constraint(generated_by_constraint_id)
                        .expect("all proofs are valid");

                    match label.as_ref() {
                        "linear_bounds" if self.propagator == Propagator::Linear => {
                            match generated_by {
                                Constraint::LinearLeq(linear) => {
                                    let mut fact = Fact {
                                        premises: inference
                                            .premises
                                            .iter()
                                            .cloned()
                                            .map(Into::into)
                                            .collect(),
                                        consequent: inference.consequent.clone().map(Into::into),
                                    };
                                    if self.run_checker {
                                        if self.check_invalid {
                                            invalidate_linear_fact(linear, &mut fact, &model);
                                        }

                                        // Setup the state for a conflict check.
                                        let variable_state =
                                            VariableState::prepare_for_conflict_check(
                                                fact.premises.clone(),
                                                fact.consequent.clone(),
                                            )
                                            .expect("Premises were inconsistent");

                                        let result = Self::verify_linear_inference(
                                            linear,
                                            &fact,
                                            variable_state,
                                        );

                                        if self.check_invalid {
                                            if result.is_ok() {
                                                return Err(CheckerError::CheckerDidNotReject {
                                                    fact: fact.clone(),
                                                    instance: self.instance,
                                                    propagator: self.propagator,
                                                    constraint: Constraint::LinearLeq(
                                                        linear.clone(),
                                                    ),
                                                });
                                            }
                                        } else {
                                            result.map_err(|_| CheckerError::CouldNotCheck {
                                                fact: fact.clone(),
                                                instance: self.instance,
                                                propagator: self.propagator,
                                                constraint: Constraint::LinearLeq(linear.clone()),
                                            })?;
                                        }
                                    }

                                    if self.check_conflicts && fact.consequent.is_none() {
                                        recreate_conflict_linear(
                                            self.instance,
                                            linear,
                                            &fact,
                                            &model,
                                        )?;
                                    }

                                    if self.check_propagations && fact.consequent.is_some() {
                                        recreate_propagation_linear(
                                            self.instance,
                                            linear,
                                            &fact,
                                            &model,
                                        )?;
                                    }
                                }
                                Constraint::LinearEq(linear) => {
                                    let inverted_linear = Linear {
                                        terms: linear
                                            .terms
                                            .iter()
                                            .map(|term| Term {
                                                weight: -term.weight,
                                                variable: term.variable.clone(),
                                            })
                                            .collect(),
                                        bound: -linear.bound,
                                    };

                                    let mut fact = Fact {
                                        premises: inference
                                            .premises
                                            .iter()
                                            .cloned()
                                            .map(Into::into)
                                            .collect(),
                                        consequent: inference.consequent.clone().map(Into::into),
                                    };

                                    // Setup the state for a conflict check.
                                    let variable_state = VariableState::prepare_for_conflict_check(
                                        fact.premises.clone(),
                                        fact.consequent.clone(),
                                    )
                                    .expect("Premises were inconsistent");

                                    if self.run_checker {
                                        let try_upper_bound = Self::verify_linear_inference(
                                            linear,
                                            &fact,
                                            variable_state.clone(),
                                        );
                                        let try_lower_bound = Self::verify_linear_inference(
                                            &inverted_linear,
                                            &fact,
                                            variable_state,
                                        );

                                        let linear = match (try_lower_bound, try_upper_bound) {
                                            (Ok(_), Ok(_)) => panic!("This should not happen."),
                                            (Ok(_), Err(_)) => &inverted_linear,
                                            (Err(_), Ok(_)) => linear,
                                            (Err(_), Err(_)) => {
                                                return Err(CheckerError::CouldNotCheck {
                                                    fact,
                                                    instance: self.instance,
                                                    propagator: self.propagator,
                                                    constraint: Constraint::LinearEq(
                                                        linear.clone(),
                                                    ),
                                                });
                                            }
                                        };

                                        if self.check_invalid {
                                            invalidate_linear_fact(linear, &mut fact, &model);

                                            // Setup the state for a conflict check.
                                            let variable_state =
                                                VariableState::prepare_for_conflict_check(
                                                    fact.premises.clone(),
                                                    fact.consequent.clone(),
                                                )
                                                .expect("Premises were inconsistent");

                                            let result = Self::verify_linear_inference(
                                                linear,
                                                &fact,
                                                variable_state,
                                            );

                                            if result.is_ok() {
                                                return Err(CheckerError::CheckerDidNotReject {
                                                    fact: fact.clone(),
                                                    instance: self.instance,
                                                    propagator: self.propagator,
                                                    constraint: Constraint::LinearLeq(
                                                        linear.clone(),
                                                    ),
                                                });
                                            }
                                        }
                                    }

                                    if self.check_conflicts && fact.consequent.is_none() {
                                        let try_upper_bound = recreate_conflict_linear(
                                            self.instance,
                                            linear,
                                            &fact,
                                            &model,
                                        );

                                        let try_lower_bound = recreate_conflict_linear(
                                            self.instance,
                                            &inverted_linear,
                                            &fact,
                                            &model,
                                        );

                                        match (try_lower_bound, try_upper_bound) {
                                            (Ok(_), Ok(_)) => panic!("This should not happen."),
                                            (Ok(_), Err(_)) | (Err(_), Ok(_)) => {}
                                            (error @ Err(_), Err(_)) => error?,
                                        }
                                    }

                                    if self.check_propagations && fact.consequent.is_some() {
                                        let try_upper_bound = recreate_propagation_linear(
                                            self.instance,
                                            linear,
                                            &fact,
                                            &model,
                                        );

                                        let try_lower_bound = recreate_propagation_linear(
                                            self.instance,
                                            &inverted_linear,
                                            &fact,
                                            &model,
                                        );

                                        match (try_lower_bound, try_upper_bound) {
                                            (Ok(_), Ok(_)) => panic!("This should not happen."),
                                            (Ok(_), Err(_)) | (Err(_), Ok(_)) => {}
                                            (error @ Err(_), Err(_)) => error?,
                                        }
                                    }
                                }
                                _ => unreachable!(),
                            }
                        }

                        "time_table" if self.propagator == Propagator::Cumulative => {
                            match generated_by {
                                Constraint::Cumulative(cumulative) => {
                                    let mut fact = Fact {
                                        premises: inference
                                            .premises
                                            .iter()
                                            .cloned()
                                            .map(Into::into)
                                            .collect(),
                                        consequent: inference.consequent.clone().map(Into::into),
                                    };

                                    if self.run_checker {
                                        if self.check_invalid {
                                            invalidate_cumulative_fact(cumulative, &mut fact);
                                        }

                                        let checker = CumulativeChecker {
                                            tasks: cumulative
                                                .tasks
                                                .iter()
                                                .map(|task| {
                                                    Task {
                                        start_time: task.start_time.clone(),
                                        duration: task
                                            .duration
                                            .try_into()
                                            .expect("Expected duration to be non-negative"),
                                        resource_usage: task
                                            .resource_usage
                                            .try_into()
                                            .expect("Expected resource usage to be non-negative"),
                                    }
                                                })
                                                .collect(),
                                            capacity: cumulative
                                                .capacity
                                                .try_into()
                                                .expect("Expected non-negative capacity"),
                                        };
                                        let variable_state =
                                            VariableState::prepare_for_conflict_check(
                                                fact.premises.clone(),
                                                fact.consequent.clone(),
                                            )
                                            .expect("Premises were inconsistent");

                                        let result = checker.check(
                                            variable_state,
                                            &fact.premises,
                                            fact.consequent.as_ref(),
                                        );

                                        if self.check_invalid {
                                            if result {
                                                return Err(CheckerError::CheckerDidNotReject {
                                                    fact: fact.clone(),
                                                    instance: self.instance,
                                                    propagator: self.propagator,
                                                    constraint: Constraint::Cumulative(
                                                        cumulative.clone(),
                                                    ),
                                                });
                                            }
                                        } else if !result {
                                            return Err(CheckerError::CouldNotCheck {
                                                fact: fact.clone(),
                                                instance: self.instance,
                                                propagator: self.propagator,
                                                constraint: Constraint::Cumulative(
                                                    cumulative.clone(),
                                                ),
                                            });
                                        }
                                    }

                                    if self.check_conflicts && fact.consequent.is_none() {
                                        recreate_conflict_cumulative(
                                            self.instance,
                                            cumulative,
                                            &fact,
                                            &model,
                                        )?;
                                    }

                                    if self.check_propagations && fact.consequent.is_some() {
                                        recreate_propagation_cumulative(
                                            self.instance,
                                            cumulative,
                                            &fact,
                                            &model,
                                        )?;
                                    }
                                }
                                _ => unreachable!(),
                            }
                        }

                        "all_different" if self.propagator == Propagator::AllDifferent => {
                            match generated_by {
                                Constraint::AllDifferent(all_different) => {
                                    let mut fact = Fact {
                                        premises: inference
                                            .premises
                                            .iter()
                                            .cloned()
                                            .map(Into::into)
                                            .collect(),
                                        consequent: inference.consequent.clone().map(Into::into),
                                    };

                                    if self.run_checker {
                                        if self.check_invalid {
                                            if fact.premises.is_empty() {
                                                continue;
                                            }
                                            invalidate_all_different_fact(all_different, &mut fact);
                                        }

                                        let checker = AllDifferentChecker {
                                            x: all_different.variables.clone(),
                                        };

                                        let variable_state =
                                            VariableState::prepare_for_conflict_check(
                                                fact.premises.clone(),
                                                fact.consequent.clone(),
                                            )
                                            .expect("Premises were inconsistent");

                                        let result = checker.check(
                                            variable_state,
                                            &fact.premises,
                                            fact.consequent.as_ref(),
                                        );

                                        if self.check_invalid {
                                            if result {
                                                return Err(CheckerError::CheckerDidNotReject {
                                                    fact: fact.clone(),
                                                    instance: self.instance,
                                                    propagator: self.propagator,
                                                    constraint: Constraint::AllDifferent(
                                                        all_different.clone(),
                                                    ),
                                                });
                                            }
                                        } else if !result {
                                            return Err(CheckerError::CouldNotCheck {
                                                fact: fact.clone(),
                                                instance: self.instance,
                                                propagator: self.propagator,
                                                constraint: Constraint::AllDifferent(
                                                    all_different.clone(),
                                                ),
                                            });
                                        }
                                    }

                                    if self.check_conflicts && fact.consequent.is_none() {
                                        recreate_conflict_all_different(
                                            self.instance,
                                            all_different,
                                            &fact,
                                            &model,
                                        )?;
                                    }

                                    if self.check_propagations && fact.consequent.is_some() {
                                        recreate_propagation_all_different(
                                            self.instance,
                                            all_different,
                                            &fact,
                                            &model,
                                        )?;
                                    }
                                }
                                _ => unreachable!(),
                            }
                        }

                        "circuit_forward_check" | "circuit_prevent"
                            if self.propagator == Propagator::Circuit =>
                        {
                            match generated_by {
                                Constraint::Circuit(circuit) => {
                                    let mut fact = Fact {
                                        premises: inference
                                            .premises
                                            .iter()
                                            .cloned()
                                            .map(Into::into)
                                            .collect(),
                                        consequent: inference.consequent.clone().map(Into::into),
                                    };

                                    if self.run_checker {
                                        if self.check_invalid {
                                            if fact.premises.is_empty() {
                                                continue;
                                            }

                                            invalidate_circuit_fact(circuit, &mut fact);
                                        }

                                        let checker = CircuitChecker {
                                            successors: circuit.successors.clone(),
                                        };

                                        let variable_state =
                                            VariableState::prepare_for_conflict_check(
                                                fact.premises.clone(),
                                                fact.consequent.clone(),
                                            )
                                            .expect("Premises were inconsistent");

                                        let result = checker.check(
                                            variable_state,
                                            &fact.premises,
                                            fact.consequent.as_ref(),
                                        );

                                        if self.check_invalid {
                                            if result {
                                                return Err(CheckerError::CheckerDidNotReject {
                                                    fact: fact.clone(),
                                                    instance: self.instance,
                                                    propagator: self.propagator,
                                                    constraint: Constraint::Circuit(
                                                        circuit.clone(),
                                                    ),
                                                });
                                            }
                                        } else if !result {
                                            return Err(CheckerError::CouldNotCheck {
                                                fact: fact.clone(),
                                                instance: self.instance,
                                                propagator: self.propagator,
                                                constraint: Constraint::Circuit(circuit.clone()),
                                            });
                                        }
                                    }

                                    if self.check_conflicts && fact.consequent.is_none() {
                                        recreate_conflict_circuit(
                                            self.instance,
                                            circuit,
                                            &fact,
                                            &model,
                                        )?;
                                    }

                                    if self.check_propagations && fact.consequent.is_some() {
                                        recreate_propagation_circuit(
                                            self.instance,
                                            circuit,
                                            &fact,
                                            &model,
                                        )?;
                                    }
                                }
                                _ => unreachable!(),
                            }
                        }

                        // Skip label and propagator combinations that we do not care
                        // about.
                        _ => {}
                    }
                }

                // Only interested in inferences.
                drcp_format::Step::Deduction(deduction) => {
                    if self.check_learned_nogoods {
                        let (solver, variables) = solver_and_variables.as_mut().unwrap();
                        recreate_deduction(&deduction, self.resolver, solver, variables);

                        let constraint_tag = solver.new_constraint_tag();
                        let clause = deduction
                            .premises
                            .iter()
                            .map(|premise| {
                                let domain = *variables.get(&premise.name).unwrap();

                                match premise.comparison {
                                    drcp_format::IntComparison::GreaterEqual => {
                                        !predicate!(domain >= premise.value)
                                    }
                                    drcp_format::IntComparison::LessEqual => {
                                        !predicate!(domain <= premise.value)
                                    }
                                    drcp_format::IntComparison::Equal => {
                                        !predicate!(domain == premise.value)
                                    }
                                    drcp_format::IntComparison::NotEqual => {
                                        !predicate!(domain != premise.value)
                                    }
                                }
                            })
                            .collect::<Vec<_>>();
                        let _ = solver.add_clause(clause, constraint_tag);
                    }

                    if self.check_deductions && !deduction.premises.is_empty() {
                        let premises = deduction
                            .premises
                            .iter()
                            .cloned()
                            .map(Atomic::IntAtomic)
                            .collect::<Vec<_>>();
                        let mut inferences = deduction
                            .sequence
                            .iter()
                            .map(|constraint_id| {
                                let fact = fact_database.get(constraint_id).unwrap_or_else(|| {
                                    panic!("Expected fact with id {constraint_id:?} to exist in database")
                                });

                                SupportingInference {
                                    premises: fact.premises.clone(),
                                    consequent: fact.consequent.clone(),
                                }
                            })
                            .collect::<Vec<_>>();

                        if self.check_invalid {
                            invalidate_nogood_deduction(&premises, &mut inferences);
                        }

                        let result = DeductionCheckerImpl.verify_deduction(premises, inferences);
                        if self.check_invalid && result {
                            return Err(CheckerError::DeductionCheckerDidNotReject {
                                deduction: deduction.clone(),
                                instance: self.instance,
                            });
                        } else if !self.check_invalid && !result {
                            return Err(CheckerError::CouldNotCheckDeduction {
                                deduction: deduction.clone(),
                                instance: self.instance,
                            });
                        }
                    }
                }
                drcp_format::Step::Conclusion(_) => {}
            }
        }

        Ok(())
    }
}

impl<'a> ProofTestRunner<'a> {
    fn verify_linear_inference(
        linear: &Linear,
        fact: &Fact,
        state: VariableState<Atomic>,
    ) -> Result<(), ()> {
        let checker = LinearChecker {
            x: linear.terms.clone(),
            bound: linear.bound,
        };
        if checker.check(state, &fact.premises, fact.consequent.as_ref()) {
            Ok(())
        } else {
            Err(())
        }
    }

    fn atomic_to_predicate(atomic: &Atomic, var: &DomainId) -> Predicate {
        match atomic {
            Atomic::IntAtomic(int_atomic) => match int_atomic.comparison {
                drcp_format::IntComparison::GreaterEqual => {
                    predicate!(var >= int_atomic.value)
                }
                drcp_format::IntComparison::LessEqual => {
                    predicate!(var <= int_atomic.value)
                }
                drcp_format::IntComparison::Equal => predicate!(var == int_atomic.value),
                drcp_format::IntComparison::NotEqual => predicate!(var != int_atomic.value),
            },
            _ => {
                unreachable!()
            }
        }
    }

    fn create_solver_for_fact(
        fact: &Fact,
        model: &Model,
    ) -> (TestSolver, HashMap<Rc<str>, DomainId>) {
        let mut solver = TestSolver::default();

        let mut variables: HashMap<Rc<str>, DomainId> = HashMap::default();

        for atomic in &fact.premises {
            let identifier = atomic.identifier();

            if !variables.contains_key(&identifier) {
                let domain = model.get_domain(&identifier);

                let var = match domain {
                    fzn_rs::ast::Domain::UnboundedInt => unimplemented!(),
                    fzn_rs::ast::Domain::Int(range_list) => solver.new_variable(
                        (*range_list.lower_bound()) as i32,
                        (*range_list.upper_bound()) as i32,
                    ),
                    fzn_rs::ast::Domain::Bool => solver.new_variable(0, 1),
                };
                let _ = variables.insert(Rc::clone(&identifier), var);
            }

            let var = variables.get(&identifier).unwrap();
            let atomic_predicate = Self::atomic_to_predicate(atomic, var);

            let _ = solver
                .post(atomic_predicate)
                .expect("Expected that apply of predicate would not lead to a conflict");
        }

        if let Some(atomic) = &fact.consequent {
            let identifier = atomic.identifier();

            if !variables.contains_key(&identifier) {
                let domain = model.get_domain(&identifier);

                let var = match domain {
                    fzn_rs::ast::Domain::UnboundedInt => unimplemented!(),
                    fzn_rs::ast::Domain::Int(range_list) => solver.new_variable(
                        (*range_list.lower_bound()) as i32,
                        (*range_list.upper_bound()) as i32,
                    ),
                    fzn_rs::ast::Domain::Bool => solver.new_variable(0, 1),
                };
                let _ = variables.insert(Rc::clone(&identifier), var);
            }
        }

        (solver, variables)
    }
}
