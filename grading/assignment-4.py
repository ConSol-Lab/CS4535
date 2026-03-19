import argparse

from common import IMPLEMENTATION_GRADE_CONTRIBUTION, build_test_cases, passed_all_test_cases

TEST_CASES_WITH_PERCENTAGE = {
    "checker_tests": 0.25,
    "processor_tests": 0.25,
}

TEST_CASES_TO_CRATE = {"checker_tests": "pumpkin-checker", "processor_tests": "pumpkin-proof-processor"}
assert sum(TEST_CASES_WITH_PERCENTAGE.values()) * 10 == IMPLEMENTATION_GRADE_CONTRIBUTION

if __name__ == "__main__":
    parser = argparse.ArgumentParser(prog="Grader Assignment 4")
    parser.add_argument(
        "--log",
        action="store_true",
        help="When this option is enabled, the names of the failed test cases are logged for debugging purposes",
    )
    args = parser.parse_args()

    print(f"Compiling checker...")
    build_test_cases(crate="pumpkin-checker")

    print(f"Compiling processor...\n")
    build_test_cases(crate="pumpkin-proof-processor")

    result = 0.0
    for test_name, grade_contribution in TEST_CASES_WITH_PERCENTAGE.items():
        if passed_all_test_cases(test_name, args.log, crate=TEST_CASES_TO_CRATE[test_name]):
            result += grade_contribution
    print("----------------------------------------------------------------------------------------")
    print()
    print(f"Final Points: {round(result, 2) * 10} out of {IMPLEMENTATION_GRADE_CONTRIBUTION}")
