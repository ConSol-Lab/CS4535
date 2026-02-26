import argparse
from pathlib import Path

if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("filename", type=Path)  # positional argument
    parser.add_argument("new_filename", type=Path)  # positional argument

    args = parser.parse_args()

    with open(args.filename, "r") as proof_file:
        with open(args.new_filename, "w+") as new_proof_file:
            for line in proof_file.readlines():
                if (
                    line.endswith("l:initial_domain\n")
                    or line.endswith("l:element\n")
                    or line.endswith("l:linear_bounds\n")
                    or line.endswith("l:nogood\n")
                ):
                    continue
                new_proof_file.write(line)
