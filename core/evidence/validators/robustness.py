"""
Robustness Validator

Evaluates model performance under controlled data perturbations
(missingness and noise) to assess operational robustness.

This module is intentionally separated from modeling code to keep
epistemic validation explicit, reproducible, and reviewable.
"""

from typing import Dict, Any, List
import numpy as np
import pandas as pd
import matplotlib.pyplot as plt

from sklearn.model_selection import StratifiedKFold, KFold
from sklearn.metrics import roc_auc_score, mean_absolute_error
from sklearn.impute import SimpleImputer
from sklearn.pipeline import Pipeline


# ---------------------------------------------------------------------
# Utilities
# ---------------------------------------------------------------------

def inject_missingness(X: pd.DataFrame, rate: float, rng: np.random.Generator):
    X_corrupt = X.copy()
    mask = rng.random(X_corrupt.shape) < rate
    X_corrupt[mask] = np.nan
    return X_corrupt


def inject_noise(X: pd.DataFrame, level: float, rng: np.random.Generator):
    if level == 0.0:
        return X
    noise = rng.normal(scale=level, size=X.shape)
    return X + noise


# ---------------------------------------------------------------------
# Core validator
# ---------------------------------------------------------------------

def run_robustness_eval(
    dataset_cfg: Dict[str, Any],
    model_cfg: Dict[str, Any],
    perturbations: Dict[str, Any],
    evaluation_cfg: Dict[str, Any],
) -> Dict[str, Any]:
    """
    Main robustness evaluation entry point.

    Returns structured metrics and optionally writes a figure.
    """

    # -----------------------------------------------------------------
    # Load dataset (placeholder hook)
    # -----------------------------------------------------------------
    X, y, task = dataset_cfg["loader"]()  
    # loader expected to return (X: DataFrame, y: Series, task: str)

    rng = np.random.default_rng(evaluation_cfg.get("random_seed", 42))

    missing_rates = perturbations["missingness"]["rates"]
    noise_levels = perturbations["noise"]["levels"]

    records: List[Dict[str, Any]] = []

    # -----------------------------------------------------------------
    # CV setup
    # -----------------------------------------------------------------
    if task == "classification":
        cv = StratifiedKFold(
            n_splits=evaluation_cfg["cross_validation"]["folds"],
            shuffle=True,
            random_state=42,
        )
        metric_fn = roc_auc_score
        metric_name = "roc_auc"
    else:
        cv = KFold(
            n_splits=evaluation_cfg["cross_validation"]["folds"],
            shuffle=True,
            random_state=42,
        )
        metric_fn = mean_absolute_error
        metric_name = "mae"

    # -----------------------------------------------------------------
    # Main perturbation loop
    # -----------------------------------------------------------------
    for miss in missing_rates:
        for noise in noise_levels:
            scores = []

            X_miss = inject_missingness(X, miss, rng)
            X_perturbed = inject_noise(X_miss, noise, rng)

            for train_idx, test_idx in cv.split(X_perturbed, y):
                X_tr, X_te = X_perturbed.iloc[train_idx], X_perturbed.iloc[test_idx]
                y_tr, y_te = y.iloc[train_idx], y.iloc[test_idx]

                pipe = Pipeline([
                    ("imputer", SimpleImputer(strategy="mean")),
                    ("model", model_cfg["estimator"]),
                ])

                pipe.fit(X_tr, y_tr)

                if task == "classification":
                    preds = pipe.predict_proba(X_te)[:, 1]
                else:
                    preds = pipe.predict(X_te)

                score = metric_fn(y_te, preds)
                scores.append(score)

            records.append({
                "missing_rate": miss,
                "noise_level": noise,
                "mean_score": float(np.mean(scores)),
                "std_score": float(np.std(scores)),
                "metric": metric_name,
            })

    metrics_df = pd.DataFrame.from_records(records)

    # -----------------------------------------------------------------
    # Optional figure (paper-style)
    # -----------------------------------------------------------------
    fig_path = "robustness_surface.svg"
    _plot_surface(metrics_df, metric_name, fig_path)

    return {
        "task": task,
        "metrics": metrics_df.to_dict(orient="records"),
        "figure_path": fig_path,
    }


# ---------------------------------------------------------------------
# Visualization
# ---------------------------------------------------------------------

def _plot_surface(df: pd.DataFrame, metric: str, path: str):
    """
    Produces a paper-style contour plot for robustness surface.
    """
    pivot = df.pivot(
        index="missing_rate",
        columns="noise_level",
        values="mean_score"
    )

    plt.figure(figsize=(6, 5))
    plt.contourf(
        pivot.columns,
        pivot.index,
        pivot.values,
        levels=15,
        alpha=0.8
    )
    plt.colorbar(label=metric)
    plt.xlabel("Noise Level")
    plt.ylabel("Missing Data Rate")
    plt.title("Model Robustness Surface")
    plt.tight_layout()
    plt.savefig(path)
    plt.close()
