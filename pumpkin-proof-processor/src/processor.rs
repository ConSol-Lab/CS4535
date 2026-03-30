//! The proof processing facilities to turn a proof scaffold into a full DRCP proof.

use std::io::BufRead;
use std::io::Write;
use std::num::NonZero;
use std::rc::Rc;
use std::sync::Arc;

use drcp_format::Conclusion;
use drcp_format::ConstraintId;
use drcp_format::Deduction;
use drcp_format::Inference;
use drcp_format::IntAtomic;
use drcp_format::IntComparison;
use drcp_format::Step;
use drcp_format::reader::ProofReader;
use drcp_format::writer::ProofWriter;
use log::debug;
use log::info;
use log::trace;
use pumpkin_core::containers::KeyedVec;
use pumpkin_core::predicate;
use pumpkin_core::predicates::Predicate;
use pumpkin_core::predicates::PredicateType;
use pumpkin_core::predicates::PropositionalConjunction;
use pumpkin_core::proof::ConstraintTag;
use pumpkin_core::proof::InferenceCode;
use pumpkin_core::state::Conflict;
use pumpkin_core::state::CurrentNogood;
use pumpkin_core::state::EmptyDomain;
use pumpkin_core::state::PropagatorConflict;
use pumpkin_core::state::PropagatorHandle;
use pumpkin_core::state::State;

use crate::deduction_propagator::DeductionPropagator;
use crate::deduction_propagator::DeductionPropagatorConstructor;
use crate::predicate_heap::PredicateHeap;
use crate::variables::Variables;

#[derive(Debug)]
pub(crate) struct ProofProcessor {
    state: State,
    variables: Variables,

    /// Contains the proof that will be written to the output. Key note: this is in reverse order
    /// due to the backward trimming.
    output_proof: Vec<ProofStage>,

    /// A queue of predicates that should still be explained.
    ///
    /// This is a re-usable heap, but its contents is specific to the processing of an
    /// individual conflict.
    to_process_heap: PredicateHeap,
}

/// A single proof stage for the proof.
#[derive(Debug)]
struct ProofStage {
    /// The ID of the constraint to introduce.
    constraint_id: ConstraintId,
    /// The inferences for the proof stage.
    inferences: Vec<Inference<String, i32, Arc<str>>>,
    /// The atomic constraints to assume `true` such that a conflict is derived throught the
    /// inferences.
    premises: Vec<IntAtomic<String, i32>>,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum ProofProcessError {
    #[error("failed to read proof: {0}")]
    Read(#[from] drcp_format::reader::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("proof did not contain a conclusion step")]
    MissingConclusion,

    #[error("undefined variable '{0}'")]
    UndefinedVariable(String),

    #[allow(unused, reason = "TODO statements will be removed in the assignment")]
    #[error("the deduction {0} does not lead to a contradiction")]
    DeductionDoesNotConflict(ConstraintId),

    #[error("all deductions together do not propagate to conflict")]
    DeductionsDoNotConflict,

    #[error("conclusion is not supported by any deduction")]
    InvalidConclusion,

    #[allow(unused, reason = "TODO statements will be removed in the assignment")]
    #[error("the deduction {0} is trivially inconsistent")]
    TriviallyInconsistentDeduction(ConstraintId),
}

#[allow(unused, reason = "Will be used in the assignment")]
/// A deduction that was posted to the solver.
#[derive(Clone, Debug)]
struct PostedDeduction {
    predicates: PropositionalConjunction,
    handle: PropagatorHandle<DeductionPropagator>,
    marked: bool,
}

type DeductionStack = KeyedVec<ConstraintTag, Option<PostedDeduction>>;

impl ProofProcessor {
    /// Creates a new [`ProofProcessor`] from a [`State`] and [`Variables`].
    ///
    /// The state is assumed to be set up with propagators. For every variable in the model, there
    /// should be an entry in `variables` mapping the variable name to a domain in `state`.
    pub(crate) fn new(state: State, variables: Variables) -> Self {
        ProofProcessor {
            state,
            variables,
            output_proof: vec![],
            to_process_heap: PredicateHeap::default(),
        }
    }

    #[allow(unused, reason = "Will be used in the assignment")]
    /// Process the given proof and write the result to the proof writer.
    ///
    /// The `proof_reader` contains the deductions in the proof.
    ///
    /// Deductions that are redundant for proving the conclusion (determined by propagation) are
    /// removed. For the deductions that remain, inferences are introduced to complete the
    /// proof.
    ///
    /// Note that processing is done based on the deductions in the proof; any inferences in the
    /// input proof are ignored.
    pub(crate) fn process<R: BufRead, W: Write>(
        mut self,
        proof_reader: ProofReader<R, i32>,
        mut proof_writer: ProofWriter<W, i32>,
    ) -> Result<(), ProofProcessError> {
        // First, we will add all the deductions to the state.
        //
        // We get the conclusion of the proof, as well as the stack of deductions (i.e., a list of
        // deductions from the proof file, it is considered a stack since we want to pop from this
        // list in reverse order of the file). The marking
        // based on the conclusion is done in the stack, so we need to look for the last marked
        // constraint and process from there.
        let (conclusion, mut deduction_stack) = self.initialise(proof_reader)?;

        let mut num_inferences = 0;

        // Next, we will start the backward trimming procedure.
        //
        // Initialization will have marked the deductions that are used to derive the
        // conclusion of the proof. So we immediately enter the backward trimming loop.
        // Going through the scaffold backwards, we remove every deduction. For each
        // deduction, if it is marked, we test whether we can derive a conflict through
        // propagation. If so, a new proof-stage is created with the appropriate
        // inferences.

        info!("Trimming deductions and determining inferences");

        // For proper logging, keep the `num_inferences` variable consistent with the
        // number of inferences introduced.
        //
        // For every deduction in the `deduction_stack`, either remove it from the final
        // proof or write a proof stage into the `output_proof`. The `output_proof` is
        // expected to be in reverse order; you should use `deduction_stack.pop()` to get the next
        // deduction, but you should not change the order besides that. Before it is written to a
        // file the proof will be reversed, but you do not need to do that.
        //
        // You cannot remove the propagators for deductions that are already processed. Instead,
        // use the [`Self::deactivate_deduction`] function to disable its propagations.
        todo!("implement proof trimming and inference introduction");

        info!(
            "Writing proof with {} stages and {} inferences",
            self.output_proof.len(),
            num_inferences,
        );

        // Finally, we write the output proof to the file. Since the proof stages are in
        // reverse order, we iterate from the end to the beginning.
        for stage in self.output_proof.drain(..).rev() {
            let mut sequence = Vec::with_capacity(stage.inferences.len());

            for inference in stage.inferences.into_iter() {
                sequence.push(inference.constraint_id);
                proof_writer.log_inference(inference)?;
            }

            proof_writer.log_deduction(Deduction {
                constraint_id: stage.constraint_id,
                premises: stage.premises,
                sequence,
            })?;
        }

        let conclusion = match conclusion {
            Conclusion::Unsat => Conclusion::Unsat,
            Conclusion::DualBound(IntAtomic {
                name,
                comparison,
                value,
            }) => Conclusion::DualBound(IntAtomic {
                name: name.as_ref().to_owned(),
                comparison,
                value,
            }),
        };

        proof_writer.log_conclusion(conclusion)?;

        info!("Proof processed successfully");

        Ok(())
    }

    #[allow(unused, reason = "Will be used in the assignment")]
    /// Deactivates the propagator in the provided [`PostedDeduction`].
    ///
    /// Also restores the [`State`] to before any deductions were posted.
    fn deactivate_deduction(&mut self, posted_deduction: &PostedDeduction) {
        let new_checkpoint = self.state.get_checkpoint() - 1;
        let _ = self.state.restore_to(new_checkpoint);
        self.state
            .get_propagator_mut(posted_deduction.handle)
            .expect("All handles should be valid")
            .deactivate();
    }

    #[allow(unused, reason = "Will be used in the assignment")]
    /// Posts all of the elements in the deduction to be true.
    fn assume_deduction(&mut self, posted_deduction: &PostedDeduction) -> Result<(), EmptyDomain> {
        for predicate in posted_deduction.predicates.iter() {
            let _ = self.state.post(*predicate)?;
        }

        Ok(())
    }

    #[allow(unused, reason = "TODO statements will be removed in the assignment")]
    /// Justifies the provided conflict.
    ///
    /// This is implemented for you since it contains a lot of pumpkin-specific code that requires
    /// knowledge of the internals.
    ///
    /// This function returns a sequence of inferences that propagate to the
    /// given conflict. The order in which the inferences are returned should allow the deduction
    /// checker to accept the deduction currently being processed.
    fn justify_conflict(
        &mut self,
        deduction_stack: &mut DeductionStack,
        conflict: Conflict,
    ) -> Vec<Inference<String, i32, Arc<str>>> {
        assert!(self.to_process_heap.is_empty());

        // The inferences in the current conflict ordered by closeness to the conflict.
        // Every element is an option, where `None` serves as a tombstone value.
        //
        // We need tombstones because initial bounds can be used multiple times within the
        // same conflict. This means when we encounter an initial bound, it may already be
        // present in the inference sequence. If that is the case, we move it to the back.
        let mut inferences: Vec<Option<Inference<String, i32, Arc<str>>>> = vec![];

        let predicates_to_explain = match conflict {
            Conflict::Propagator(PropagatorConflict {
                conjunction,
                inference_code,
            }) => {
                let generated_by = inference_code.tag();
                let label = inference_code.label();

                inferences.push(Some(Inference {
                    constraint_id: self.state.new_constraint_tag().into(),
                    premises: convert_predicates_to_proof_atomic(&self.variables, &conjunction),
                    consequent: None,
                    generated_by: Some(generated_by.into()),
                    label: Some(label.into()),
                }));

                mark_stack_entry(deduction_stack, inference_code);

                conjunction
            }
            Conflict::EmptyDomain(empty_domain_confict) => {
                let mut predicates_to_explain = vec![];
                empty_domain_confict.get_reason(
                    &mut self.state,
                    &mut predicates_to_explain,
                    CurrentNogood::empty(),
                );

                let inference_code = empty_domain_confict.trigger_inference_code;

                let generated_by = inference_code.tag();
                let label = inference_code.label();

                inferences.push(Some(Inference {
                    constraint_id: self.state.new_constraint_tag().into(),
                    premises: convert_predicates_to_proof_atomic(
                        &self.variables,
                        &predicates_to_explain,
                    ),
                    consequent: Some(convert_predicate_to_proof_atomic(
                        &self.variables,
                        empty_domain_confict.trigger_predicate,
                    )),
                    generated_by: Some(generated_by.into()),
                    label: Some(label.into()),
                }));

                mark_stack_entry(deduction_stack, inference_code);

                predicates_to_explain.push(!empty_domain_confict.trigger_predicate);

                predicates_to_explain.into()
            }
        };

        // Then we add the conflict to the queue of predicates that need to be explained
        // through inferences.
        for predicate in predicates_to_explain {
            self.to_process_heap.push(predicate, &self.state);
        }

        self.introduce_inferences(deduction_stack, inferences)
    }

    #[allow(unused, reason = "TODO statements will be removed in the assignment")]
    /// Introduces the inferences for the elements in `self.to_process_heap` by adding them to the
    /// provided `inferences`.
    ///
    /// Note that you also need to introduce inferences for the initial bounds. While these are
    /// implicitly present in the solver, they need to explicitly added to the proof.
    fn introduce_inferences(
        &mut self,
        deduction_stack: &mut DeductionStack,
        mut inferences: Vec<Option<Inference<String, i32, Arc<str>>>>,
    ) -> Vec<Inference<String, i32, Arc<str>>> {
        // Functions to use:
        // - `PredicateHeap::push`
        // - `State::is_implied_by_initial_domain`
        // - `State::new_constraint_tag`
        // - `Self::get_propagation_reason`
        // - `convert_predicates_to_proof_atomic` and `convert_predicate_to_proof_atomic`
        // - `mark_entry`
        //
        // The label for initial domain inferences is created by:
        // ```
        // Arc::from("initial_domain")
        // ```

        // For every predicate in the queue, we will introduce appropriate inferences into
        // the proof.
        while let Some(predicate) = self.to_process_heap.pop() {
            todo!("explain why {predicate:?} is true")
        }

        // Reverse the inference sequence, since analysis goes from the conflict to the
        // leafs rather than from the assumptions to the conflict. Also, filter out the
        // tombstone values.
        inferences.into_iter().rev().flatten().collect()
    }

    /// Returns the reason for the provided `Predicate` and, if available, its inference code.
    ///
    /// Note that the inference code will be [`None`] for explicitly-trailed predicates.
    #[allow(unused, reason = "TODO statements will be removed in the assignment")]
    fn get_propagation_reason(
        &mut self,
        predicate: Predicate,
    ) -> (Vec<Predicate>, Option<InferenceCode>) {
        let mut reason = vec![];
        let inference_code =
            self.state
                .get_propagation_reason(predicate, &mut reason, CurrentNogood::empty());

        (reason.into(), inference_code)
    }

    /// Initialise the state with the deductions from the proof, returning either an error or the
    /// conclusion of the proof and a list of the deductions in the proof.
    ///
    /// This method also marks the deductions responsible for finding the conclusion.
    fn initialise<R: BufRead>(
        &mut self,
        mut proof_reader: ProofReader<R, i32>,
    ) -> Result<(Conclusion<Rc<str>, i32>, DeductionStack), ProofProcessError> {
        info!("Setting up solver with deductions");

        let mut deduction_stack = KeyedVec::default();

        let mut num_deductions = 0;

        if let Err(conflict) = self.state.propagate_to_fixed_point() {
            // If the state is in a conflict after propagating only the model
            // constraints, the proof consists of only one proof stage.

            let empty_nogood_tag = self.state.new_constraint_tag();
            deduction_stack.accomodate(empty_nogood_tag, None);

            let inferences = self.justify_conflict(&mut deduction_stack, conflict);

            num_deductions += 1;

            info!("Processing proof scaffold with {num_deductions} deductions");

            // Log the empty clause to the proof.
            self.output_proof.push(ProofStage {
                inferences,
                constraint_id: empty_nogood_tag.into(),
                premises: vec![],
            });

            return Ok((Conclusion::Unsat, deduction_stack));
        }

        loop {
            // Try to read the next step from the proof.
            let next_step = proof_reader.next_step()?;

            // If we reached the end of the proof, it is incomplete.
            let Some(step) = next_step else {
                return Err(ProofProcessError::MissingConclusion);
            };

            // Extract the deduction from the step, or otherwise handle the step type.
            let deduction = match step {
                Step::Deduction(deduction) => {
                    num_deductions += 1;
                    deduction
                }

                // A dual bound conclusion is encountered. This is of the form `true -> bound`.
                // This means we have to find a nogood `!bound -> false` and mark it.
                Step::Conclusion(Conclusion::DualBound(bound)) => {
                    info!("Processing proof scaffold with {num_deductions} deductions");
                    return self.prepare_trimming_for_dual_bound_conclusion(bound, deduction_stack);
                }

                Step::Conclusion(Conclusion::Unsat) => {
                    return Err(ProofProcessError::DeductionsDoNotConflict);
                }

                // We ignore inferences when doing proof processing. We will introduce our own
                // inferences regardless of what is in the input.
                Step::Inference(_) => continue,
            };

            // Get the constraint tag for this new constraint.
            //
            // To make comparing the scaffold with the full proof easier, we want to
            // match up the constraint IDs of the deductions. Hence, we generate
            // constraint IDs until the one we get is the same as observed in the
            // scaffold.
            let mut constraint_tag = self.state.new_constraint_tag();
            while NonZero::from(constraint_tag) < deduction.constraint_id {
                constraint_tag = self.state.new_constraint_tag();
            }
            assert_eq!(ConstraintId::from(constraint_tag), deduction.constraint_id);

            // Convert the deduction to the solver's representation.
            let nogood = PropositionalConjunction::from(
                deduction
                    .premises
                    .iter()
                    .map(|premise| convert_proof_atomic_to_predicate(&self.variables, premise))
                    .collect::<Result<Vec<_>, ProofProcessError>>()?,
            );

            debug!("Adding deduction {}", deduction.constraint_id);
            trace!("    {nogood:?}");

            self.state.new_checkpoint();
            let handle = self.state.add_propagator(DeductionPropagatorConstructor {
                nogood: nogood.iter().copied().collect(),
                constraint_tag,
            });

            deduction_stack.accomodate(constraint_tag, None);
            deduction_stack[constraint_tag] = Some(PostedDeduction {
                predicates: nogood,
                handle,
                marked: false,
            });

            if let Err(conflict) = self.state.propagate_to_fixed_point() {
                trace!("Conflict identified");

                // If we reach inconsistency through propagation alone, then it must mean that the
                // proof is a proof of unsatisfiability and not a dual bound proof.
                info!("Processing proof scaffold with {num_deductions} deductions");
                return self.repair_solver(conflict, constraint_tag, deduction_stack);
            }
        }
    }

    fn prepare_trimming_for_dual_bound_conclusion(
        &mut self,
        bound: IntAtomic<Rc<str>, i32>,
        mut deduction_stack: DeductionStack,
    ) -> Result<(Conclusion<Rc<str>, i32>, DeductionStack), ProofProcessError> {
        let predicate = convert_proof_atomic_to_predicate(&self.variables, &bound)?;
        info!("Found dual bound conclusion");
        trace!("bound = {predicate:?}");

        // If the claimed bound is not true given the current assignment, then the
        // conclusion does not follow by propagation.
        if self.state.truth_value(predicate) != Some(true) {
            return Err(ProofProcessError::InvalidConclusion);
        }

        // If the dual bound is the initial bound on the objective variable, write the
        // correct proof in that situation and short-circuit.
        if self.state.is_implied_by_initial_domain(predicate) {
            self.to_process_heap.push(predicate, &self.state);

            let inferences = self.introduce_inferences(&mut deduction_stack, vec![]);
            let deduction_id = self.state.new_constraint_tag();

            self.output_proof.push(ProofStage {
                inferences,
                constraint_id: deduction_id.into(),
                premises: vec![convert_predicate_to_proof_atomic(
                    &self.variables,
                    !predicate,
                )],
            });

            return Ok((Conclusion::DualBound(bound), deduction_stack));
        }

        let mut reason_buffer = vec![];
        let inference_code = self.state.get_propagation_reason(
            predicate,
            &mut reason_buffer,
            CurrentNogood::empty(),
        );

        // We do not use the function to mark the constraint in the nogood stack. It
        // could happen that the conclusion is a root bound, but the proof does not
        // contain a nogood asserting the root bound (an inference is not enough, we
        // explicitly want a deduction that makes the conclusion true).
        trace!("Marking reason for dual bound");
        trace!("  reason = {reason_buffer:?}",);

        // We expect the conclusion to be syntactally implied by one of the deductions.
        let used_constraint_tag = inference_code.expect("must be due to a propagation").tag();
        trace!("  constraint_tag = {}", NonZero::from(used_constraint_tag));

        let Some(stack_entry) = deduction_stack
            .get_mut(used_constraint_tag)
            .map(|opt| opt.as_mut())
        else {
            return Err(ProofProcessError::InvalidConclusion);
        };

        if let Some(posted_deduction) = stack_entry {
            posted_deduction.marked = true;
        } else {
            // In this case we have to explain by 'root propagation' and no deductions
            // were used. The predicate is propagated by a propagator.
            self.to_process_heap.push(predicate, &self.state);
            let inferences = self.introduce_inferences(&mut deduction_stack, vec![]);
            let deduction_id = self.state.new_constraint_tag();

            self.output_proof.push(ProofStage {
                inferences,
                constraint_id: deduction_id.into(),
                premises: vec![convert_predicate_to_proof_atomic(
                    &self.variables,
                    !predicate,
                )],
            });
        }

        Ok((Conclusion::DualBound(bound), deduction_stack))
    }

    /// Takes a solver that is in an inconsistent state because we have a proof of
    /// unsatisfiability, and removes the last added constraint. We mark all the deductions in the
    /// deduction stack that contributed to the conflict, and log the empty nogood step.
    fn repair_solver(
        &mut self,
        conflict: Conflict,
        tag: ConstraintTag,
        mut deduction_stack: DeductionStack,
    ) -> Result<(Conclusion<Rc<str>, i32>, DeductionStack), ProofProcessError> {
        // Add the nogood that triggered the conflict to the nogood stack. Also mark it,
        // because obviously it is used.
        let posted_deduction = deduction_stack[tag]
            .as_mut()
            .expect("the deduction that triggered the conflict must be on the nogood stack");
        posted_deduction.marked = true;

        let inferences = self.justify_conflict(&mut deduction_stack, conflict);

        // Log the empty clause to the proof.
        self.output_proof.push(ProofStage {
            inferences,
            constraint_id: self.state.new_constraint_tag().into(),
            premises: vec![],
        });

        Ok((Conclusion::Unsat, deduction_stack))
    }
}

#[allow(unused, reason = "Will be used in the assignment")]
/// Given a [`DeductionStack`], mark the constraint indicated by the inference code as used.
fn mark_stack_entry(stack: &mut DeductionStack, inference_code: InferenceCode) {
    let used_constraint_tag = inference_code.tag();

    trace!("Marking constraint {}", NonZero::from(used_constraint_tag));

    let stack_entry = &mut stack[used_constraint_tag];
    if let Some(posted_deduction) = stack_entry {
        posted_deduction.marked = true;
    }
}

#[allow(unused, reason = "Will be used in the assignment")]
/// Converts a slice of predicates to atomic constraints for the proof log.
fn convert_predicates_to_proof_atomic(
    variables: &Variables,
    predicates: &[Predicate],
) -> Vec<IntAtomic<String, i32>> {
    predicates
        .iter()
        .map(|predicate| convert_predicate_to_proof_atomic(variables, *predicate))
        .collect()
}

/// Converts an atomic from the proofs representation to a solver [`Predicate`].
fn convert_predicate_to_proof_atomic(
    variables: &Variables,
    predicate: Predicate,
) -> IntAtomic<String, i32> {
    let name = variables
        .get_name_for_domain(predicate.get_domain())
        .unwrap()
        .as_ref()
        .to_owned();

    let comparison = match predicate.get_predicate_type() {
        PredicateType::LowerBound => IntComparison::GreaterEqual,
        PredicateType::UpperBound => IntComparison::LessEqual,
        PredicateType::NotEqual => IntComparison::NotEqual,
        PredicateType::Equal => IntComparison::Equal,
    };

    let value = predicate.get_right_hand_side();

    IntAtomic {
        name,
        comparison,
        value,
    }
}

/// Converts an atomic from the proofs representation to a solver [`Predicate`].
fn convert_proof_atomic_to_predicate(
    variables: &Variables,
    atomic: &IntAtomic<Rc<str>, i32>,
) -> Result<Predicate, ProofProcessError> {
    let domain_id = variables
        .get_domain_by_name(&atomic.name)
        .ok_or_else(|| ProofProcessError::UndefinedVariable(atomic.name.as_ref().to_owned()))?;

    let predicate = match atomic.comparison {
        IntComparison::GreaterEqual => {
            predicate![domain_id >= atomic.value]
        }
        IntComparison::LessEqual => {
            predicate![domain_id <= atomic.value]
        }
        IntComparison::Equal => predicate![domain_id == atomic.value],
        IntComparison::NotEqual => {
            predicate![domain_id != atomic.value]
        }
    };

    Ok(predicate)
}

#[cfg(test)]
mod tests {
    use drcp_format::IntComparison::*;
    use drcp_format::reader::ReadAtomic;
    use drcp_format::reader::ReadStep;
    use pumpkin_propagators::arithmetic::BinaryEqualsPropagatorArgs;

    use super::*;

    #[test]
    fn dual_bound_is_initial_bound() {
        let mut state = State::default();
        let mut variables = Variables::default();
        let x1 = state.new_interval_variable(10, 20, Some("x1".into()));
        variables.add_variable("x1".into(), x1);

        let scaffold = r#"
            a 1 [x1 <= 9]
            n 1 1 0
            c -1
        "#;

        let expected = vec![
            inference(
                2,
                [],
                Some(atomic("x1", GreaterEqual, 10)),
                None,
                Some("initial_domain"),
            ),
            deduction(3, [atomic("x1", LessEqual, 9)], [2]),
            Step::Conclusion(Conclusion::DualBound(atomic("x1", GreaterEqual, 10))),
        ];

        test_processing(state, variables, scaffold, expected);
    }

    #[test]
    fn dual_bound_is_root_propagation() {
        let mut state = State::default();
        let mut variables = Variables::default();
        let x1 = state.new_interval_variable(0, 20, Some("x1".into()));
        variables.add_variable("x1".into(), x1);
        let x2 = state.new_interval_variable(10, 10, Some("x2".into()));
        variables.add_variable("x2".into(), x2);

        let constraint_tag = state.new_constraint_tag();
        let _ = state.add_propagator(BinaryEqualsPropagatorArgs {
            a: x1,
            b: x2,
            constraint_tag,
        });

        let scaffold = r#"
            a 1 [x1 <= 9]
            n 2 1 0
            c -1
        "#;

        let expected = vec![
            inference(
                4,
                [],
                Some(atomic("x2", GreaterEqual, 10)),
                None,
                Some("initial_domain"),
            ),
            inference(
                3,
                [atomic("x2", GreaterEqual, 10)],
                Some(atomic("x1", GreaterEqual, 10)),
                Some(1),
                Some("binary_equals"),
            ),
            deduction(5, [atomic("x1", LessEqual, 9)], [4, 3]),
            Step::Conclusion(Conclusion::DualBound(atomic("x1", GreaterEqual, 10))),
        ];

        test_processing(state, variables, scaffold, expected);
    }

    fn inference(
        constraint_id: u32,
        premises: impl Into<Vec<ReadAtomic<i32>>>,
        consequent: Option<ReadAtomic<i32>>,
        generated_by: Option<u32>,
        label: Option<&str>,
    ) -> ReadStep<i32> {
        Step::Inference(Inference {
            constraint_id: NonZero::new(constraint_id).expect("constraint_id is non-zero"),
            premises: premises.into(),
            consequent,
            generated_by: generated_by
                .map(|id| NonZero::new(id).expect("constraint_id is non-zero")),
            label: label.map(Rc::from),
        })
    }

    fn deduction(
        constraint_id: u32,
        premises: impl Into<Vec<ReadAtomic<i32>>>,
        sequence: impl IntoIterator<Item = u32>,
    ) -> ReadStep<i32> {
        Step::Deduction(Deduction {
            constraint_id: NonZero::new(constraint_id).expect("constraint_id is non-zero"),
            premises: premises.into(),
            sequence: sequence
                .into_iter()
                .map(|id| NonZero::new(id).expect("constraint_id is non-zero"))
                .collect(),
        })
    }

    fn atomic(name: &str, comparison: IntComparison, value: i32) -> ReadAtomic<i32> {
        IntAtomic {
            name: Rc::from(name),
            comparison,
            value,
        }
    }

    fn test_processing(
        state: State,
        variables: Variables,
        scaffold: &str,
        expected: Vec<ReadStep<i32>>,
    ) {
        let mut processed = Vec::new();

        let processor = ProofProcessor::new(state, variables);
        processor
            .process(
                ProofReader::new(scaffold.as_bytes()),
                ProofWriter::new(&mut processed),
            )
            .expect("successful process");

        let mut processed_reader = ProofReader::<_, i32>::new(processed.as_slice());
        let processed_proof = std::iter::from_fn(|| {
            processed_reader
                .next_step()
                .expect("processor returns valid proof")
        })
        .collect::<Vec<_>>();

        assert_eq!(processed_proof, expected);
    }
}
