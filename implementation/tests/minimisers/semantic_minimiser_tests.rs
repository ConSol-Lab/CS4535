#![cfg(test)]
#![allow(deprecated, reason = "Solver can be used in the course.")]
use implementation::conflict_analysis::SemanticMinimiser;
use pumpkin_core::Solver;
use pumpkin_core::branching::Brancher;
use pumpkin_core::branching::BrancherEvent;
use pumpkin_core::branching::SelectionContext;
use pumpkin_core::conflict_resolving::NogoodMinimiser;
use pumpkin_core::conjunction;
use pumpkin_core::predicate;
use pumpkin_core::predicates::Predicate;
use pumpkin_core::predicates::PropositionalConjunction;

#[derive(Debug, Default)]
struct DummyBrancher;

impl Brancher for DummyBrancher {
    fn next_decision(&mut self, _context: &mut SelectionContext) -> Option<Predicate> {
        todo!()
    }

    fn subscribe_to_events(&self) -> Vec<BrancherEvent> {
        todo!()
    }
}

#[test]
fn trivial_nogood() {
    let mut p = SemanticMinimiser::new();
    let mut solver = Solver::default();
    let domain_id = solver.new_bounded_integer(0, 10);
    let mut nogood: Vec<Predicate> = vec![predicate!(domain_id >= 0), predicate!(domain_id <= 10)];

    let mut brancher = DummyBrancher;
    p.minimise(
        &mut solver.conflict_analysis_context(&mut brancher),
        &mut nogood,
    );

    assert!(nogood.is_empty());
}

#[test]
fn trivial_conflict_bounds() {
    let mut p = SemanticMinimiser::new();
    let mut solver = Solver::default();
    let domain_id = solver.new_bounded_integer(0, 10);
    let mut nogood: Vec<Predicate> = vec![predicate!(domain_id >= 5), predicate!(domain_id <= 4)];

    let mut brancher = DummyBrancher;
    p.minimise(
        &mut solver.conflict_analysis_context(&mut brancher),
        &mut nogood,
    );

    assert_eq!(nogood.len(), 1);
    assert_eq!(nogood[0], Predicate::trivially_false());
}

#[test]
fn trivial_conflict_holes() {
    let mut p = SemanticMinimiser::new();
    let mut solver = Solver::default();
    let domain_id = solver.new_bounded_integer(0, 10);
    let mut nogood: Vec<Predicate> = vec![
        predicate!(domain_id != 5),
        predicate!(domain_id >= 5),
        predicate!(domain_id <= 5),
    ];

    let mut brancher = DummyBrancher;
    p.minimise(
        &mut solver.conflict_analysis_context(&mut brancher),
        &mut nogood,
    );

    assert_eq!(nogood.len(), 1);
    assert_eq!(nogood[0], Predicate::trivially_false());
}

#[test]
fn trivial_conflict_assignment() {
    let mut p = SemanticMinimiser::new();
    let mut solver = Solver::default();
    let domain_id = solver.new_bounded_integer(0, 10);
    let mut nogood: Vec<Predicate> = vec![predicate!(domain_id != 5), predicate!(domain_id == 5)];

    let mut brancher = DummyBrancher;
    p.minimise(
        &mut solver.conflict_analysis_context(&mut brancher),
        &mut nogood,
    );

    assert_eq!(nogood.len(), 1);
    assert_eq!(nogood[0], Predicate::trivially_false());
}

#[test]
fn trivial_conflict_bounds_reset() {
    let mut p = SemanticMinimiser::new();
    let mut solver = Solver::default();
    let domain_id = solver.new_bounded_integer(0, 10);
    let mut nogood: Vec<Predicate> = vec![predicate!(domain_id != 5), predicate!(domain_id == 5)];

    let mut brancher = DummyBrancher;
    p.minimise(
        &mut solver.conflict_analysis_context(&mut brancher),
        &mut nogood,
    );

    let mut brancher = DummyBrancher;
    let mut other = Vec::default();
    p.minimise(
        &mut solver.conflict_analysis_context(&mut brancher),
        &mut other,
    );

    assert!(other.is_empty());
}

#[test]
fn simple_bound1() {
    let mut p = SemanticMinimiser::new();
    let mut solver = Solver::default();
    let domain_0 = solver.new_bounded_integer(0, 10);
    let domain_1 = solver.new_bounded_integer(0, 5);

    let mut nogood: Vec<Predicate> =
        conjunction!([domain_0 >= 5] & [domain_0 <= 9] & [domain_1 >= 0] & [domain_1 <= 4]).into();

    let mut brancher = DummyBrancher;
    p.minimise(
        &mut solver.conflict_analysis_context(&mut brancher),
        &mut nogood,
    );

    assert_eq!(nogood.len(), 3);
    assert_eq!(
        PropositionalConjunction::from(nogood),
        conjunction!([domain_0 >= 5] & [domain_0 <= 9] & [domain_1 <= 4])
    );
}

#[test]
fn simple_bound2() {
    let mut p = SemanticMinimiser::new();
    let mut solver = Solver::default();
    let domain_0 = solver.new_bounded_integer(0, 10);
    let domain_1 = solver.new_bounded_integer(0, 5);

    let mut nogood = conjunction!(
        [domain_0 >= 5] & [domain_0 <= 9] & [domain_1 >= 0] & [domain_1 <= 4] & [domain_0 != 7]
    )
    .into();

    let mut brancher = DummyBrancher;
    p.minimise(
        &mut solver.conflict_analysis_context(&mut brancher),
        &mut nogood,
    );

    assert_eq!(nogood.len(), 4);
    assert_eq!(
        PropositionalConjunction::from(nogood),
        conjunction!([domain_0 >= 5] & [domain_0 <= 9] & [domain_1 <= 4] & [domain_0 != 7])
    );
}

#[test]
fn simple_bound3() {
    let mut p = SemanticMinimiser::new();
    let mut solver = Solver::default();
    let domain_0 = solver.new_bounded_integer(0, 10);
    let domain_1 = solver.new_bounded_integer(0, 5);

    let mut nogood = conjunction!(
        [domain_0 >= 5]
            & [domain_0 <= 9]
            & [domain_1 >= 0]
            & [domain_1 <= 4]
            & [domain_0 != 7]
            & [domain_0 != 7]
            & [domain_0 != 8]
            & [domain_0 != 6]
    )
    .into();

    let mut brancher = DummyBrancher;
    p.minimise(
        &mut solver.conflict_analysis_context(&mut brancher),
        &mut nogood,
    );

    assert_eq!(nogood.len(), 6);
    assert_eq!(
        PropositionalConjunction::from(nogood),
        conjunction!(
            [domain_0 >= 5]
                & [domain_0 <= 9]
                & [domain_1 <= 4]
                & [domain_0 != 7]
                & [domain_0 != 6]
                & [domain_0 != 8]
        )
    );
}

#[test]
fn simple_assign() {
    let mut p = SemanticMinimiser::new();
    let mut solver = Solver::default();
    let domain_0 = solver.new_bounded_integer(0, 10);
    let domain_1 = solver.new_bounded_integer(0, 5);

    let mut nogood = conjunction!(
        [domain_0 >= 5]
            & [domain_0 <= 9]
            & [domain_1 >= 0]
            & [domain_1 <= 4]
            & [domain_0 != 7]
            & [domain_0 != 7]
            & [domain_0 != 6]
            & [domain_0 == 5]
            & [domain_0 != 7]
    )
    .into();

    let mut brancher = DummyBrancher;
    p.minimise(
        &mut solver.conflict_analysis_context(&mut brancher),
        &mut nogood,
    );

    assert_eq!(nogood.len(), 2);
    assert_eq!(
        PropositionalConjunction::from(nogood),
        conjunction!([domain_0 == 5] & [domain_1 <= 4])
    );
}

#[test]
fn simple_lb_override1() {
    let mut p = SemanticMinimiser::new();
    let mut solver = Solver::default();
    let domain_0 = solver.new_bounded_integer(0, 10);

    let mut nogood = conjunction!([domain_0 >= 2] & [domain_0 >= 1] & [domain_0 >= 5]).into();

    let mut brancher = DummyBrancher;
    p.minimise(
        &mut solver.conflict_analysis_context(&mut brancher),
        &mut nogood,
    );

    assert_eq!(nogood.len(), 1);
    assert_eq!(nogood[0], predicate!(domain_0 >= 5));
}

#[test]
fn hole_lb_override() {
    let mut p = SemanticMinimiser::new();
    let mut solver = Solver::default();
    let domain_0 = solver.new_bounded_integer(0, 10);

    let mut nogood =
        conjunction!([domain_0 != 2] & [domain_0 != 3] & [domain_0 >= 5] & [domain_0 >= 1]).into();

    let mut brancher = DummyBrancher;
    p.minimise(
        &mut solver.conflict_analysis_context(&mut brancher),
        &mut nogood,
    );

    assert_eq!(nogood.len(), 1);
    assert_eq!(
        PropositionalConjunction::from(nogood),
        conjunction!([domain_0 >= 5])
    );
}

#[test]
fn hole_push_lb() {
    let mut p = SemanticMinimiser::new();
    let mut solver = Solver::default();
    let domain_0 = solver.new_bounded_integer(0, 10);

    let mut nogood =
        conjunction!([domain_0 != 2] & [domain_0 != 3] & [domain_0 >= 1] & [domain_0 != 1]).into();

    let mut brancher = DummyBrancher;
    p.minimise(
        &mut solver.conflict_analysis_context(&mut brancher),
        &mut nogood,
    );

    assert_eq!(nogood.len(), 1);
    assert_eq!(nogood[0], predicate![domain_0 >= 4]);
}
