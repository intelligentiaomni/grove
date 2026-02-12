#!/usr/bin/env python3
"""
Robustness evaluation pipeline for CS/Math datasets
Generates metrics + figures for classification tasks
"""

import json
import numpy as np
import pandas as pd
import matplotlib.pyplot as plt
import pathlib

from sklearn.datasets import load_iris, load_digits, load_wine
from sklearn.impute import SimpleImputer
from sklearn.model_selection import StratifiedKFold
from sklearn.pipeline import Pipeline
from sklearn.ensemble import RandomForestClassifier
from sklearn.metrics import roc_auc_score
from datetime import datetime
from pathlib import Path

# -----------------------------
# Helpers
# -----------------------------

def inject_noise(X: pd.DataFrame, level: float = 0.1):
    return X + np.random.normal(0, level, X.shape)

def inject_missingness(X: pd.DataFrame, missing_rate: float = 0.1):
    X_missing = X.copy()
    n_missing = int(missing_rate * X.size)
    idx = np.random.choice(X.size, n_missing, replace=False)
    X_missing.values.flat[idx] = np.nan
    return X_missing

def detect_threshold_crossings(scores, thresholds, *, direction="higher_is_better"):
    results = {}
    for t in thresholds:
        crossings = [i for i, s in enumerate(scores)
                     if (s >= t and direction=="higher_is_better")
                     or (s <= t and direction=="lower_is_better")]
        results[t] = crossings
    return results

# -----------------------------
# Dataset configurations
# -----------------------------

dataset_cfgs = [
    {
        "name": "iris",
        "loader": lambda: (load_iris(as_frame=True).data,
                           load_iris(as_frame=True).target,
                           "classification")
    },
    {
        "name": "digits",
        "loader": lambda: (pd.DataFrame(load_digits(as_frame=True).data),
                           load_digits(as_frame=True).target,
                           "classification")
    },
    {
        "name": "wine",
        "loader": lambda: (load_wine(as_frame=True).data,
                           load_wine(as_frame=True).target,
                           "classification")
    }
]

# -----------------------------
# Main pipeline
# -----------------------------

def run_robustness_eval(output_dir: Path):
    output_dir.mkdir(parents=True, exist_ok=True)
    figures_dir = output_dir / "figures"
    figures_dir.mkdir(exist_ok=True)

    all_metrics = []

    for cfg in dataset_cfgs:
        X, y, task = cfg["loader"]()

        # Simple pipeline: imputer + RF
        model = Pipeline([
            ("imputer", SimpleImputer(strategy="mean")),
            ("clf", RandomForestClassifier(n_estimators=10, random_state=42))
        ])

        # Sweep missingness and noise
        missing_rates = [0.0, 0.1, 0.2]
        noise_levels = [0.0, 0.05, 0.1]

        for m in missing_rates:
            for n in noise_levels:
                X_mod = inject_missingness(X, m)
                X_mod = inject_noise(X_mod, n)

                # 3-fold Stratified CV
                cv = StratifiedKFold(n_splits=3, shuffle=True, random_state=42)
                scores = []
                for train_idx, test_idx in cv.split(X_mod, y):
                    model.fit(X_mod.iloc[train_idx], y.iloc[train_idx])
                    probs = model.predict_proba(X_mod.iloc[test_idx])
                    scores.append(
                        roc_auc_score(
                            y.iloc[test_idx],
                            probs,
                            multi_class="ovr"
                        )
                    )

                mean_score = float(np.mean(scores))

                # Append metrics
                all_metrics.append({
                    "dataset": cfg["name"],
                    "metric": "auc",
                    "mean_score": mean_score,
                    "missing_rate": m,
                    "noise_level": n
                })

        # Plot heatmap per dataset
        df = pd.DataFrame([m for m in all_metrics if m["dataset"] == cfg["name"]])
        pivot = df.pivot_table(index="missing_rate", columns="noise_level", values="mean_score")
        plt.figure(figsize=(4,3))
        plt.imshow(pivot, origin="lower", aspect="auto")
        plt.colorbar(label="Mean AUC")
        plt.xlabel("Noise level")
        plt.ylabel("Missing rate")
        plt.title(f"Robustness - {cfg['name']}")
        fig_path = figures_dir / f"robustness_{cfg['name']}.svg"
        plt.tight_layout()
        plt.savefig(fig_path)
        plt.close()

    # Save all metrics
    metrics_path = output_dir / "metrics.json"
    with open(metrics_path, "w") as f:
        json.dump(all_metrics, f, indent=2)

    print(f"[INFO] Metrics saved to {metrics_path}")
    print(f"[INFO] Figures saved to {figures_dir}")

# -----------------------------
# Entry point
# -----------------------------

if __name__ == "__main__":
    run_robustness_eval(Path("experiments/pipelines/robustness-eval/results/example_run"))
