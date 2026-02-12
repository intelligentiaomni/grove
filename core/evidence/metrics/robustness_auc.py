"""
Robustness AUC Metric

Defines ROC-AUC–based robustness metrics for classification models
under data perturbations.

This module isolates metric semantics from validation logic to keep
epistemic assumptions explicit and reusable.
"""

from typing import Dict, Any, List
import pandas as pd


# ---------------------------------------------------------------------
# Metric computation
# ---------------------------------------------------------------------

def summarize_auc_robustness(
    metrics: List[Dict[str, Any]]
) -> Dict[str, Any]:
    """
    Summarize robustness behavior for ROC-AUC metrics.

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

    if df["metric"].iloc[0] != "roc_auc":
        raise ValueError("robustness_auc called on non-AUC metrics.")

    # -----------------------------------------------------------------
    # Aggregate summaries
    # -----------------------------------------------------------------
    summary = {
        "metric": "roc_auc",
        "mean_auc": float(df["mean_score"].mean()),
        "min_auc": float(df["mean_score"].min()),
        "max_auc": float(df["mean_score"].max()),
        "std_auc": float(df["mean_score"].std()),
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
        df.sort_values("mean_score")
        .iloc[0]
        .to_dict()
    )

    summary["worst_case"] = {
        "missing_rate": worst_case["missing_rate"],
        "noise_level": worst_case["noise_level"],
        "roc_auc": worst_case["mean_score"],
    }

    return summary
