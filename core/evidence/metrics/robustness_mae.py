"""
Robustness MAE Metric

Defines MAE-based robustness metrics for regression models
under data perturbations.

Lower values indicate better performance and robustness.
"""

from typing import Dict, Any, List
import pandas as pd


# ---------------------------------------------------------------------
# Metric computation
# ---------------------------------------------------------------------

def summarize_mae_robustness(
    metrics: List[Dict[str, Any]]
) -> Dict[str, Any]:
    """
    Summarize robustness behavior for MAE metrics.

    Parameters
    ----------
    metrics : list of dict
        Output records from robustness evaluation.

    Returns
    -------
    dict
        Aggregate robustness statistics.
    """

    df = pd.DataFrame(metrics)

    if df["metric"].iloc[0] != "mae":
        raise ValueError("robustness_mae called on non-MAE metrics.")

    # -----------------------------------------------------------------
    # Aggregate summaries
    # -----------------------------------------------------------------
    summary = {
        "metric": "mae",
        "mean_mae": float(df["mean_score"].mean()),
        "min_mae": float(df["mean_score"].min()),
        "max_mae": float(df["mean_score"].max()),
        "std_mae": float(df["mean_score"].std()),
    }

    # -----------------------------------------------------------------
    # Sensitivity to perturbations
    # -----------------------------------------------------------------
    by_missing = (
        df.groupby("missing_rate")["mean_score"]
        .mean()
        .to_dict()
    )

    by_noise = (
        df.groupby("noise_level")["mean_score"]
        .mean()
        .to_dict()
    )

    summary["sensitivity"] = {
        "by_missing_rate": by_missing,
        "by_noise_level": by_noise,
    }

    # -----------------------------------------------------------------
    # Worst-case regime (operational pessimism)
    # -----------------------------------------------------------------
    worst_case = (
        df.sort_values("mean_score", ascending=False)
        .iloc[0]
        .to_dict()
    )

    summary["worst_case"] = {
        "missing_rate": worst_case["missing_rate"],
        "noise_level": worst_case["noise_level"],
        "mae": worst_case["mean_score"],
    }

    return summary
