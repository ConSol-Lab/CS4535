# Pumpkin for CS4535

Pumpkin is a combinatorial optimisation solver developed by the ConSol Lab at TU Delft. It is based on the (lazy clause generation) constraint programming paradigm.

This repository contains a modified version of Pumpkin that is used for the Constraint Solving course at TU Delft (CS4535).

# Usage

Since Pumpkin is written in pure Rust, it is easy to install! After cloning, you can build the project using a version of [Rust](https://www.rust-lang.org/tools/install) (1.72.1+) using the following commands:

```sh
cargo build           # Creates a non-optimized build with debug info
cargo build --release # Creates an optimized build
```

## MiniZinc
Pumpkin serves as a backend solver for the [MiniZinc](https://www.minizinc.org/) modelling language.

To use it as such a backend, follow the following steps:

- Step 1: Clone the repository and build it using `cargo build --release`.
- Step 2: Install MiniZinc using the [appropriate executable](https://www.minizinc.org/resources/) or [binary archive](https://www.minizinc.org/downloads/).
- Step 3: Add the following to the `MZN_SOLVER_PATH` environment variable: `<path_to_pumpkin>/minizinc` (see [this thread](https://askubuntu.com/questions/58814/how-do-i-add-environment-variables) on how to do this using a shell).
- Step 4: Check whether the installation worked using the command `minizinc --help pumpkin`.

## Components
Pumpkin consists of 3 different crates:

- The library contained in [core](https://github.com/ConSol-Lab/Pumpkin/tree/main/pumpkin-crates/core); defines the API through which the solver can be used via Rust.
- The CLI contained in [pumpkin-solver](https://github.com/ConSol-Lab/Pumpkin/tree/main/pumpkin-solver/src/bin/pumpkin-solver); defines the usage of Pumpkin through a command line.
- The proof logging contained in [drcp-format](https://github.com/ConSol-Lab/Pumpkin/tree/main/drcp-format); defines proof logging which can be used in combination with Pumpkin.
- The python bindings contained in [pumpkin-solver-py](https://github.com/ConSol-Lab/Pumpkin/tree/main/pumpkin-solver-py); defines the python interface for Pumpkin

The easiest way to get to know the different modules is through the documentation. This documentation can be created automatically using the command:
```sh
cargo doc --no-deps
```

## Examples
There are several examples of how to use the solver specified in the documentation of the different components. For more concrete examples of how to use Pumpkin to solve a set of example problems, we refer to the [examples folder](https://github.com/ConSol-Lab/Pumpkin/tree/main/pumpkin-solver/examples) which contains examples such as bibd, nqueens, and disjunctive scheduling.

## Documentation
One of the development goals of Pumpkin is to ensure that the solver is easy-to-use and well-documented. To this end, it is required that any external contribution is well-documented (both the structs/enums/methods and the implementation itself)!

## Formatting
To make use of these formatting rules, we require the [nightly toolchain](https://doc.rust-lang.org/beta/book/appendix-07-nightly-rust.html). *Note that we only use the nightly toolchain for formatting.* The nightly version which can be installed using the following command:
```sh
rustup toolchain install --component rustfmt -- nightly
```
The formatting can then be run using:
```sh
cargo +nightly fmt
```
