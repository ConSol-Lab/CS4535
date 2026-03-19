#![cfg(test)] // workaround for https://github.com/rust-lang/rust-clippy/issues/11024
use paste::paste;

macro_rules! accept_proof {
    ($name:ident) => {
        paste! {
            #[test]
            fn [<valid_ $name>]() {
                run_checker_on_proof(stringify!($name));
            }
        }
    };
}

macro_rules! reject_proof {
    ($name:ident) => {
        paste! {
            #[test]
            fn [<invalid_ $name>]() {
                run_checker_on_proof_invalid(stringify!($name));
            }
        }
    };
}

accept_proof!(market_split_u3_01);
accept_proof!(market_split_u3_02);
accept_proof!(market_split_u3_03);
accept_proof!(market_split_u3_04);

reject_proof!(market_split_u3_01);
reject_proof!(market_split_u3_02);
reject_proof!(market_split_u3_03);
reject_proof!(market_split_u3_04);

fn run_checker_on_proof(model: &str) {
    let model_path = format!(
        "{}/tests/valid_proofs/{model}.fzn",
        env!("CARGO_MANIFEST_DIR")
    );
    let proof_path = format!(
        "{}/tests/valid_proofs/{model}.drcp",
        env!("CARGO_MANIFEST_DIR")
    );

    let checker_status = escargot::CargoBuild::new()
        .package("pumpkin-checker")
        .bin("pumpkin-checker")
        .current_target()
        .current_release()
        .run()
        .unwrap()
        .command()
        .arg(model_path)
        .arg(proof_path)
        .output()
        .unwrap();

    assert!(
        checker_status.status.success(),
        "Checker did not succeed:\n{}",
        str::from_utf8(&checker_status.stderr).expect("Expected to be able to decode error status")
    );
}

fn run_checker_on_proof_invalid(model: &str) {
    let model_path = format!(
        "{}/tests/invalid_proofs/{model}.fzn",
        env!("CARGO_MANIFEST_DIR")
    );
    let proof_path = format!(
        "{}/tests/invalid_proofs/{model}.drcp",
        env!("CARGO_MANIFEST_DIR")
    );

    let checker_status = escargot::CargoBuild::new()
        .package("pumpkin-checker")
        .bin("pumpkin-checker")
        .current_target()
        .current_release()
        .run()
        .unwrap()
        .command()
        .arg(model_path)
        .arg(proof_path)
        .output()
        .unwrap();

    assert!(
        !checker_status.status.success()
            && checker_status
                .status
                .code()
                .expect("Expected exit code from process")
                != 101,
        "Checker should return an error without panicking:\n{}",
        str::from_utf8(&checker_status.stderr).expect("Expected to be able to decode error status")
    );
}
