import logging
from pathlib import Path
from typing import Tuple

import click
import matplotlib.pyplot as plt
import polars as pl
from matplotlib.ticker import LogLocator

from constraint_solving import common_init
from constraint_solving.config import Config
from constraint_solving.experiments import Experiment, ExperimentDoesNotExist, load_experiment


def load_csv(config: Config, experiment_name: str) -> Tuple[Experiment, Path]:
    try:
        experiment = load_experiment(config, experiment_name)
    except ExperimentDoesNotExist:
        logging.error(f"Experiment '{experiment_name}' does not exist.")
        raise Exception()

    results_dir = config.results_dir / experiment.name

    if not results_dir.is_dir():
        logging.error(f"Experiment `{experiment_name}` does not have a results dir")
        raise Exception()

    results_csv = results_dir / f"run_data_{experiment.name}.csv"
    if not results_csv.is_file():
        logging.error(f"Experiment {experiment_name} does not have a run_data file")
        raise Exception()

    return (experiment, results_csv)


def generate_image(
    config: Config,
    experiment_name: str,
    other_experiment_name: str,
    combined: pl.DataFrame,
    column_name: str,
    max_x_lim: None | int = None,
    max_y_lim: None | int = None,
    log: bool = True,
):
    image_dir = config.figures_dir / f"{experiment_name}__{other_experiment_name}"
    image_dir.mkdir(parents=True, exist_ok=True)

    if column_name in combined.columns and f"{column_name}_right" in combined.columns:
        plt.scatter(combined[column_name], combined[f"{column_name}_right"])

        max_x_lim = max_x_lim if max_x_lim is not None else combined[f"{column_name}"].max()
        max_y_lim = max_y_lim if max_y_lim is not None else combined[f"{column_name}_right"].max()

        max_lim = 1.05 * max(max_y_lim, max_x_lim)

        if log and (combined[column_name] > 0).any() and (combined[f"{column_name}_right"] > 0).any():
            plt.xlim(1e-1, max_lim)
            plt.ylim(1e-1, max_lim)

            plt.plot([1e-1, max_lim], [1e-1, max_lim], linestyle="--", color="red")

            plt.xscale("log")
            plt.yscale("log")

            locator = LogLocator(base=10)

            ax = plt.gca()
            ax.xaxis.set_major_locator(locator)
            ax.yaxis.set_major_locator(locator)
        else:
            plt.xlim(0, max_lim)
            plt.ylim(0, max_lim)

            plt.plot([0, max_lim], [0, max_lim], linestyle="--", color="red")

            if log:
                logging.info(
                    f"Could not find any values `> 0` for {column_name} of {experiment_name}; not applying log scale"
                )

        plt.gca().set_aspect("equal", adjustable="box")
    else:
        logging.warning(
            f"Column '{column_name}' was not found in the statistics; this indicates that either {experiment_name} or"
            f" {other_experiment_name} did not contain any statistics."
        )
    plt.xlabel(f"{column_name} {experiment_name}")
    plt.ylabel(f"{column_name} {other_experiment_name}")
    plt.title(f"{column_name} comparison")
    plt.savefig(image_dir / f"{column_name}.png")

    plt.clf()


@click.command()
@click.argument("experiment_name", type=str)
@click.argument("other_experiment_name", type=str)
@click.option("--proof", is_flag=True)
def run(experiment_name: str, other_experiment_name: str, proof: bool) -> int:
    config = common_init()

    # Load first experiment
    try:
        (_, experiment_csv) = load_csv(config, experiment_name)
    except Exception as e:
        print(e)
        return 1

    # Load other experiment
    try:
        (_, other_experiment_csv) = load_csv(config, other_experiment_name)
    except Exception as e:
        print(e)
        return 1

    logging.info("Processing...")

    pl.Config.set_tbl_rows(-1)
    pl.Config.set_tbl_hide_column_data_types(True)
    pl.Config.set_tbl_hide_dataframe_shape(True)

    experiment_df = pl.read_csv(experiment_csv)
    if not proof:
        status_overview = experiment_df.group_by("status").count()
        print(f"Instance statuses {experiment_name}:\n{status_overview}\n")

    other_experiment_df = pl.read_csv(other_experiment_csv)
    if not proof:
        other_status_overview = other_experiment_df.group_by("status").count()
        print(f"Instance statuses {other_experiment_name}:\n{other_status_overview}")

    if not proof:
        combined = experiment_df.join(other_experiment_df, on="benchmark-key").filter(
            pl.col("status") == "OPTIMAL", pl.col("status_right") == "OPTIMAL"
        )

        generate_image(config, experiment_name, other_experiment_name, combined, "failures")
        generate_image(
            config,
            experiment_name,
            other_experiment_name,
            combined,
            "solveTime",
            max_x_lim=150,
            max_y_lim=150,
            log=False,
        )
    else:
        (optimisation_df, feasibility_df) = (
            (experiment_df, other_experiment_df)
            if experiment_df.select(pl.col("benchmark-key").str.contains("rcpsp").all()).item()
            else (other_experiment_df, experiment_df)
        )
        image_dir = config.figures_dir / f"{experiment_name}__{other_experiment_name}"
        image_dir.mkdir(parents=True, exist_ok=True)

        plt.scatter(
            optimisation_df["Scaffold #Deductions"], optimisation_df["Processed #Deductions"], label="Optimisation"
        )
        plt.scatter(
            feasibility_df["Scaffold #Deductions"],
            feasibility_df["Processed #Deductions"],
            label="Feasibility",
            marker="s",
        )

        max_x_lim = max(optimisation_df["Scaffold #Deductions"].max(), feasibility_df["Scaffold #Deductions"].max())
        max_y_lim = max(optimisation_df["Processed #Deductions"].max(), feasibility_df["Processed #Deductions"].max())

        max_lim = 1.05 * max(max_y_lim, max_x_lim)
        plt.xlim(1e-1, max_lim)
        plt.ylim(1e-1, max_lim)

        plt.plot([1e-1, max_lim], [1e-1, max_lim], linestyle="--", color="red")

        plt.xscale("log")
        plt.yscale("log")

        locator = LogLocator(base=10)

        ax = plt.gca()
        ax.xaxis.set_major_locator(locator)
        ax.yaxis.set_major_locator(locator)

        plt.gca().set_aspect("equal", adjustable="box")

        plt.xlabel("#Deductions Scaffold")
        plt.ylabel("#Deductions Processed")
        plt.title("#Deductions comparison")

        plt.legend(loc="upper left")

        plt.savefig(image_dir / "num_deductions.png")

        plt.clf()

        print()
        print("TABLE:\n")

        headers = ["Instance", "\\#Deductions Scaffold", "\\#Deductions Processed", "\\#Inferences"]

        textabular = f"|c{'|c|'*len(headers)}"
        texheader = "\\hline\n" + " & ".join(headers) + "\\\\"
        texdata = "\\hline\n\\hline\n"

        texdata += "\\multicolumn{4}{|c|}{\\textbf{Feasibility Instances}}\\\\"
        for row in (
            feasibility_df.select(
                ["benchmark-key", "Scaffold #Deductions", "Processed #Deductions", "Processed #Inferences"]
            )
            .sort("benchmark-key")
            .iter_rows(named=True)
        ):
            texdata += "\\hline\n"
            texdata += (
                " & ".join(
                    str(value).replace("_", "\\_") if isinstance(value, str) else f"{value:,d}"
                    for value in row.values()
                )
                + "\\\\\n"
            )

        texdata += "\\hline\n"
        texdata += "\\multicolumn{4}{|c|}{\\textbf{Optimisation Instances}}\\\\"
        for row in (
            optimisation_df.select(
                ["benchmark-key", "Scaffold #Deductions", "Processed #Deductions", "Processed #Inferences"]
            )
            .sort("benchmark-key")
            .iter_rows(named=True)
        ):
            texdata += "\\hline\n"
            texdata += (
                " & ".join(
                    str(value).replace("_", "\\_") if isinstance(value, str) else f"{value:,d}"
                    for value in row.values()
                )
                + "\\\\\n"
            )
        texdata += "\\hline\n"

        print("\\begin{tabular}{" + textabular + "}")
        print(texheader)
        print(texdata, end="")
        print("\\end{tabular}")
    return 0
