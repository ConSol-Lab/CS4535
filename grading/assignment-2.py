import argparse

from common import IMPLEMENTATION_GRADE_CONTRIBUTION, build_test_cases, passed_all_test_cases

TEST_CASES_WITH_PERCENTAGE = {
    "propagators::circuit_tests::circuit_propagation_tests": 0.10,
    "propagators::circuit_tests::circuit_conflict_tests": 0.05,
    "propagators::circuit_tests::circuit_checker_tests": 0.10,
    "propagators::all_different_tests::all_different_propagation_tests": 0.10,
    "propagators::all_different_tests::all_different_conflict_tests": 0.05,
    "propagators::all_different_tests::all_different_checker_tests": 0.10,
}
assert sum(TEST_CASES_WITH_PERCENTAGE.values()) * 10 == IMPLEMENTATION_GRADE_CONTRIBUTION

if __name__ == "__main__":
    parser = argparse.ArgumentParser(prog="Grader Assignment 2")
    parser.add_argument(
        "--log",
        action="store_true",
        help="When this option is enabled, the names of the failed test cases are logged for debugging purposes",
    )
    args = parser.parse_args()

    print(f"Compiling...\n")
    build_test_cases()

    result = 0.0
    for test_name, grade_contribution in TEST_CASES_WITH_PERCENTAGE.items():
        if passed_all_test_cases(test_name, args.log):
            result += grade_contribution
    print("----------------------------------------------------------------------------------------")
    print()
    print(f"Final Points: {round(result, 2) * 10} out of {IMPLEMENTATION_GRADE_CONTRIBUTION}")
