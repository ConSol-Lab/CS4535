import argparse

from common import (
    ENDC,
    FAIL,
    IMPLEMENTATION_GRADE_CONTRIBUTION,
    build_test_cases,
    passed_all_test_cases,
    percentage_passed,
)

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
    if passed_all_test_cases("checker_tests", args.log, crate=TEST_CASES_TO_CRATE["checker_tests"]):
        result += TEST_CASES_WITH_PERCENTAGE["checker_tests"]

    percentage_passed_valid = percentage_passed(
        "checker_tests::valid", False, crate=TEST_CASES_TO_CRATE["checker_tests"]
    )
    percentage_passed_invalid = percentage_passed(
        "checker_tests::invalid", False, crate=TEST_CASES_TO_CRATE["checker_tests"]
    )

    if percentage_passed_valid < 0.5 or percentage_passed_invalid < 0.5:
        print(
            f"{FAIL}Failed either more than 50% of the valid or of the invalid test cases of the checker\n\tPercentage"
            f" passed valid: {percentage_passed_valid * 100}%\n\tPercentaged passed invalid:"
            f" {percentage_passed_invalid * 100}%{ENDC}"
        )
    elif passed_all_test_cases("processor_tests", args.log, crate=TEST_CASES_TO_CRATE["processor_tests"], timeout=90):
        result += TEST_CASES_WITH_PERCENTAGE["processor_tests"]

    print("----------------------------------------------------------------------------------------")
    print()
    print(f"Final Points: {round(result, 2) * 10} out of {IMPLEMENTATION_GRADE_CONTRIBUTION}")
