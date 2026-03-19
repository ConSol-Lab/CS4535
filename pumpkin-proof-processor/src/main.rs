mod deduction_propagator;
mod model;
mod predicate_heap;
mod processor;
mod variables;

use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::rc::Rc;

use clap::Parser;
use clap_verbosity_flag::InfoLevel;
use clap_verbosity_flag::Verbosity;
use drcp_format::reader::ProofReader;
use drcp_format::writer::ProofWriter;
use implementation::propagators::cumulative::CumulativeConstructor;
use implementation::propagators::cumulative::Task;
use implementation::propagators::linear::LinearConstructor;
use pumpkin_core::containers::StorageKey;
use pumpkin_core::proof::ConstraintTag;
use pumpkin_core::state::State;
use pumpkin_core::variables::TransformableVariable;
use pumpkin_propagators::arithmetic::BinaryNotEqualsPropagatorArgs;

use crate::model::FlatZincConstraints;
use crate::model::FlatZincModel;
use crate::processor::ProofProcessor;
use crate::variables::Variables;

#[derive(Parser)]
struct Cli {
    #[command(flatten)]
    verbose: Verbosity<InfoLevel>,

    /// Path to the model file (.fzn).
    model_path: PathBuf,

    /// Path to the scaffold file.
    scaffold_path: PathBuf,

    /// Path to the full proof (output).
    full_proof_path: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    env_logger::Builder::new()
        .format_timestamp(None)
        .format_target(false)
        .filter_level(cli.verbose.log_level_filter())
        .target(env_logger::Target::Stdout)
        .init();

    let proof_processor = parse_model(&cli.model_path)?;
    let proof_reader = create_proof_reader(&cli.scaffold_path)?;
    let proof_writer = create_proof_writer(&cli.full_proof_path)?;

    proof_processor.process(proof_reader, proof_writer)?;

    Ok(())
}

fn parse_model(path: impl AsRef<Path>) -> anyhow::Result<ProofProcessor> {
    let model_source = std::fs::read_to_string(path)?;

    // TODO: For now the error handling shortcuts here. Ideally the `FznError` type returns
    // something that can be converted to an owned type, but for now we have to work around the
    // error holding a reference to the source.
    let fzn_ast = fzn_rs::fzn::parse(&model_source).map_err(|err| anyhow::anyhow!("{err}"))?;

    let fzn_model = FlatZincModel::from_ast(fzn_ast)?;

    let mut state = State::default();
    let mut variable_map = Variables::default();

    // Create all the flatzinc variables in the solver.
    for (name, variable) in fzn_model.variables.iter() {
        let domain_id = model::create_domain_for_variable(&mut state, name, variable);
        variable_map.add_variable(Rc::clone(name), domain_id);
    }

    for (idx, annotated_constraint) in fzn_model.constraints.iter().enumerate() {
        let constraint_tag = state.new_constraint_tag();
        assert_eq!(
            idx,
            constraint_tag.index(),
            "constraint tags for model constraints must be consecutive"
        );

        // If the solver is no-longer consistent, we still want to reserve constraint tags for the
        // remaining flatzinc constraints. The processor expects the empty nogood to have a
        // constraint tag that follows after _all_ the flatzinc constraints have one.

        post_constraint(
            &fzn_model,
            &mut state,
            &mut variable_map,
            constraint_tag,
            &annotated_constraint.constraint.node,
        )?;
    }

    Ok(ProofProcessor::new(state, variable_map))
}

/// Post the constraint to the state.
fn post_constraint(
    fzn_model: &FlatZincModel,
    state: &mut State,
    variable_map: &mut Variables,
    constraint_tag: ConstraintTag,
    constraint: &FlatZincConstraints,
) -> anyhow::Result<()> {
    match constraint {
        FlatZincConstraints::LinearLeq {
            weights,
            variables,
            bound,
        } => {
            let weights = fzn_model.resolve_array(weights)?;
            let variables = fzn_model.resolve_array(variables)?;

            let mut terms = vec![];

            for (weight, variable) in weights.zip(variables) {
                let weight = weight?;
                let variable = variable_map.resolve(variable?, state);

                terms.push(variable.scaled(weight));
            }

            let _ = state.add_propagator(LinearConstructor {
                x: terms.into(),
                bound: *bound,
                constraint_tag,
                conflict_detection_only: false,
            });
        }

        FlatZincConstraints::LinearEq {
            weights,
            variables,
            bound,
        } => {
            let weights = fzn_model.resolve_array(weights)?;
            let variables = fzn_model.resolve_array(variables)?;

            let mut terms = vec![];

            for (weight, variable) in weights.zip(variables) {
                let weight = weight?;
                let variable = variable_map.resolve(variable?, state);

                terms.push(variable.scaled(weight));
            }

            let negated_terms = terms.iter().map(|var| var.scaled(-1)).collect::<Box<[_]>>();

            let _ = state.add_propagator(LinearConstructor {
                x: negated_terms,
                bound: -*bound,
                constraint_tag,
                conflict_detection_only: false,
            });

            let _ = state.add_propagator(LinearConstructor {
                x: terms.into(),
                bound: *bound,
                constraint_tag,
                conflict_detection_only: false,
            });
        }

        FlatZincConstraints::Cumulative {
            start_times,
            durations,
            resource_usages,
            capacity,
        } => {
            let start_times = fzn_model
                .resolve_array(start_times)?
                .map(|variable| {
                    let domain_id = variable_map.resolve(variable?, state);
                    Ok(domain_id)
                })
                .collect::<Result<Vec<_>, fzn_rs::InstanceError>>()?;
            let durations = fzn_model
                .resolve_array(durations)?
                .collect::<Result<Vec<_>, _>>()?;
            let resource_usages = fzn_model
                .resolve_array(resource_usages)?
                .collect::<Result<Vec<_>, _>>()?;

            let tasks = start_times
                .into_iter()
                .zip(durations)
                .zip(resource_usages)
                .map(|((start_time, duration), resource_usage)| Task {
                    start_time,
                    duration: u32::try_from(duration).unwrap(),
                    resource_usage: u32::try_from(resource_usage).unwrap(),
                })
                .collect();

            let _ = state.add_propagator(CumulativeConstructor {
                tasks,
                capacity: u32::try_from(*capacity).unwrap(),
                constraint_tag,
                conflict_detection_only: false,
            });
        }

        FlatZincConstraints::AllDifferent(variables) => {
            let variables = fzn_model
                .resolve_array(variables)?
                .map(|variable| {
                    let domain_id = variable_map.resolve(variable?, state);
                    Ok(domain_id)
                })
                .collect::<Result<Vec<_>, fzn_rs::InstanceError>>()?;

            for i in 0..variables.len() {
                for j in i + 1..variables.len() {
                    let _ = state.add_propagator(BinaryNotEqualsPropagatorArgs {
                        a: variables[i],
                        b: variables[j],
                        constraint_tag,
                    });
                }
            }
        }
    }

    Ok(())
}

fn create_proof_reader(
    path: impl AsRef<Path>,
) -> anyhow::Result<ProofReader<Box<dyn BufRead>, i32>> {
    let file = File::open(path.as_ref())?;

    if path.as_ref().extension().is_some_and(|ext| ext == "gz") {
        let decoder = flate2::read::GzDecoder::new(file);
        let buf_reader = BufReader::new(decoder);

        Ok(ProofReader::new(Box::new(buf_reader)))
    } else {
        let buf_reader = BufReader::new(file);

        Ok(ProofReader::new(Box::new(buf_reader)))
    }
}

fn create_proof_writer(path: impl AsRef<Path>) -> anyhow::Result<ProofWriter<Box<dyn Write>, i32>> {
    let file = File::create(path.as_ref())?;

    if path.as_ref().extension().is_some_and(|ext| ext == "gz") {
        let encoder = flate2::write::GzEncoder::new(file, flate2::Compression::fast());
        let buf_writer = BufWriter::new(encoder);
        Ok(ProofWriter::new(Box::new(buf_writer)))
    } else {
        let buf_writer = BufWriter::new(file);
        Ok(ProofWriter::new(Box::new(buf_writer)))
    }
}
