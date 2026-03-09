use std::rc::Rc;

use drcp_format::Deduction;
use implementation::conflict_analysis::AllDecisionResolver;
use implementation::conflict_analysis::DeductionCheckerImpl;
use implementation::conflict_analysis::OneUIP;
use implementation::conflict_analysis::SemanticMinimiser;
use pumpkin_checking::AtomicConstraint;
use pumpkin_checking::VariableState;
use pumpkin_core::Random;
use pumpkin_core::Solver;
use pumpkin_core::branching::Brancher;
use pumpkin_core::conflict_resolving::Atomic;
use pumpkin_core::conflict_resolving::SupportingInference;
use pumpkin_core::constraints::Constraint as _;
use pumpkin_core::containers::HashMap;
use pumpkin_core::options::ConflictResolverType;
use pumpkin_core::options::SolverOptions;
use pumpkin_core::predicate;
use pumpkin_core::predicates::Predicate;
#[allow(clippy::disallowed_types, reason = "Used for the assignment")]
use pumpkin_core::rand::SeedableRng;
use pumpkin_core::rand::rngs::SmallRng;
use pumpkin_core::results::SatisfactionResultUnderAssumptions;
use pumpkin_core::termination::Indefinite;
use pumpkin_core::variables::TransformableVariable;

use crate::Constraint;
use crate::Model;

mod all_decision_resolver_tests;
mod deduction_checker_tests;
mod resolution_resolver_tests;

pub(crate) fn recreate_deduction(
    deduction: &Deduction<Rc<str>, i32>,
    model: &Model,
    resolver: ConflictResolverType,
) {
    let mut solver = Solver::with_options_and_minimiser(
        SolverOptions {
            should_minimise_nogoods: false,
            ..Default::default()
        },
        Box::new(SemanticMinimiser::new()),
        Box::new(DeductionCheckerImpl),
    );
    let mut variables = HashMap::new();
    for (name, domain) in model.iter_domains() {
        if !variables.contains_key(name) {
            let domain_id = match domain {
                fzn_rs::ast::Domain::UnboundedInt => todo!(),
                fzn_rs::ast::Domain::Int(range_list) => solver.new_named_bounded_integer(
                    *range_list.lower_bound() as i32,
                    *range_list.upper_bound() as i32,
                    name.to_string(),
                ),
                fzn_rs::ast::Domain::Bool => {
                    solver.new_named_bounded_integer(0, 1, name.to_string())
                }
            };
            let _ = variables.insert(Rc::clone(name), domain_id);
        }
    }

    for (_, constraint) in model.iter_constraints() {
        let constraint_tag = solver.new_constraint_tag();
        match constraint {
            Constraint::Nogood(_) => todo!(),
            Constraint::LinearLeq(linear) => {
                let vars = linear
                    .terms
                    .iter()
                    .map(|term| {
                        match &term.variable.0 {
                            fzn_rs::VariableExpr::Identifier(id) => *variables.get(id).unwrap(),
                            fzn_rs::VariableExpr::Constant(constant) => {
                                solver.new_bounded_integer(*constant, *constant)
                            }
                        }
                        .scaled(term.weight.into())
                    })
                    .collect::<Vec<_>>();
                let result = pumpkin_constraints::less_than_or_equals(
                    vars,
                    linear.bound,
                    constraint_tag,
                    false,
                )
                .post(&mut solver);
                if result.is_err() {
                    panic!("Problem is UNSAT")
                }
            }
            Constraint::LinearEq(linear) => {
                let vars = linear
                    .terms
                    .iter()
                    .map(|term| {
                        match &term.variable.0 {
                            fzn_rs::VariableExpr::Identifier(id) => *variables.get(id).unwrap(),
                            fzn_rs::VariableExpr::Constant(constant) => {
                                solver.new_bounded_integer(*constant, *constant)
                            }
                        }
                        .scaled(term.weight.into())
                    })
                    .collect::<Vec<_>>();
                let result = pumpkin_constraints::equals(vars, linear.bound, constraint_tag, false)
                    .post(&mut solver);

                if result.is_err() {
                    panic!("Problem is UNSAT")
                }
            }
            Constraint::Cumulative(_cumulative) => todo!(),
            Constraint::AllDifferent(_all_different) => todo!(),
            Constraint::Circuit(_circuit) => todo!(),
            Constraint::Element(_element) => todo!(),
        }
    }

    let assumptions = deduction
        .premises
        .iter()
        .map(|premise| {
            let domain = *variables.get(&premise.name).unwrap();

            match premise.comparison {
                drcp_format::IntComparison::GreaterEqual => {
                    predicate!(domain >= premise.value)
                }
                drcp_format::IntComparison::LessEqual => {
                    predicate!(domain <= premise.value)
                }
                drcp_format::IntComparison::Equal => {
                    predicate!(domain == premise.value)
                }
                drcp_format::IntComparison::NotEqual => {
                    predicate!(domain != premise.value)
                }
            }
        })
        .collect::<Vec<_>>();
    let mut brancher = DummyBrancher;
    match resolver {
        ConflictResolverType::NoLearning => todo!(),
        ConflictResolverType::UIP => {
            let mut resolver = OneUIP::new();
            let result = solver.satisfy_under_assumptions(
                &mut brancher,
                &mut Indefinite,
                &mut resolver,
                &assumptions,
            );
            assert!(matches!(
                result,
                SatisfactionResultUnderAssumptions::UnsatisfiableUnderAssumptions(_)
            ));
        }
        ConflictResolverType::AllDecision => {
            let mut resolver = AllDecisionResolver::new();
            let result = solver.satisfy_under_assumptions(
                &mut brancher,
                &mut Indefinite,
                &mut resolver,
                &assumptions,
            );

            assert!(matches!(
                result,
                SatisfactionResultUnderAssumptions::UnsatisfiableUnderAssumptions(_)
            ));
        }
    }
}

pub(crate) fn invalidate_nogood_deduction(
    premises: &[Atomic],
    inferences: &mut [SupportingInference],
) {
    if premises.is_empty() {
        return;
    }

    let mut rng = SmallRng::seed_from_u64((premises.len() + inferences.len()) as u64);

    loop {
        let mut state = VariableState::default();
        for premise in premises.iter() {
            let _ = state.apply(premise);
        }

        let index = rng.generate_usize_in_range(0..inferences.len());
        for inference in inferences[0..index].iter() {
            if let Some(consequent) = &inference.consequent {
                let _ = state.apply(consequent);
            }
        }

        let inference = &mut inferences[index];
        if inference.premises.is_empty() {
            continue;
        }

        if inference
            .premises
            .iter()
            .all(|atomic| state.is_true(atomic))
        {
            let predicate_index = rng.generate_usize_in_range(0..inference.premises.len());

            let atomic = &mut inference.premises[predicate_index];

            let id = atomic.identifier();
            let comparison = atomic.comparison();

            match comparison {
                pumpkin_checking::Comparison::GreaterEqual
                | pumpkin_checking::Comparison::Equal => {
                    atomic.set_value(state.lower_bound(&id).as_int().unwrap() + 1);
                }
                pumpkin_checking::Comparison::LessEqual => {
                    atomic.set_value(state.upper_bound(&id).as_int().unwrap() - 1)
                }
                pumpkin_checking::Comparison::NotEqual => {
                    atomic.set_value(state.lower_bound(&id).as_int().unwrap());
                }
            }

            assert!(
                !inference
                    .premises
                    .iter()
                    .all(|atomic| state.is_true(atomic))
            );

            break;
        }
    }
}

struct DummyBrancher;

impl Brancher for DummyBrancher {
    fn next_decision(
        &mut self,
        _context: &mut pumpkin_core::branching::SelectionContext,
    ) -> Option<Predicate> {
        None
    }

    fn subscribe_to_events(&self) -> Vec<pumpkin_core::branching::BrancherEvent> {
        vec![]
    }
}
