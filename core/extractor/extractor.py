import json
from pathlib import Path
import shutil
import random

# ----------------------------
# Config / Paths
# ----------------------------
ram_folder = Path("RAM")
log_folder = Path("Logs")
theory_folder = Path("Theory")
extractor_folder = Path("Extractor")
population_file = extractor_folder / "population.json"
history_file = extractor_folder / "history.csv"

log_folder.mkdir(exist_ok=True, parents=True)
theory_folder.mkdir(exist_ok=True, parents=True)

# ----------------------------
# Load extractor population
# ----------------------------
with open(population_file) as f:
    population = json.load(f)

# Pick first extractor instance for simplicity
extractor = population[0]

theta_rec = extractor["θ_rec"]
theta_imp = extractor["θ_imp"]
theta_theory = extractor["θ_theory"]

# ----------------------------
# Novelty tracking (placeholder)
# ----------------------------
# Collect text of previous Logs/Theory for simple novelty check
def collect_previous_text():
    texts = []
    for folder in [log_folder, theory_folder]:
        for f in folder.rglob("*.txt"):
            texts.append(f.read_text())
    return texts

previous_texts = collect_previous_text()

def is_novel(text, prev_texts, threshold=0.2):
    """Simple novelty proxy: fraction of unseen words"""
    words = set(text.split())
    seen_words = set()
    for pt in prev_texts:
        seen_words.update(pt.split())
    unseen = words - seen_words
    return (len(unseen)/max(1,len(words))) >= threshold

# ----------------------------
# Minimal scoring functions
# ----------------------------
def compute_rec(text):
    return 1 if "repeat" in text else 0

def compute_act(text):
    return 1 if any(k in text for k in ["run", "test", "measure"]) else 0

def compute_imp(text):
    return 1 if "important" in text else 0

def compute_form(text):
    return 1 if "=" in text else 0

# ----------------------------
# Evolutionary mutation (minimal)
# ----------------------------
def mutate_threshold(value, sigma=0.05):
    """Add small random variation to threshold"""
    return max(0.0, min(1.0, value + random.uniform(-sigma, sigma)))

# ----------------------------
# Extraction Loop
# ----------------------------
cycle_stats = {"RAM":0, "Logs":0, "Theory":0, "Novel":0}

for ram_file in ram_folder.rglob("*.txt"):
    with open(ram_file) as f:
        text = f.read()
    cycle_stats["RAM"] += 1

    rec = compute_rec(text)
    act = compute_act(text)
    imp = compute_imp(text)
    form = compute_form(text)

    novel = is_novel(text, previous_texts)
    if novel:
        cycle_stats["Novel"] += 1

    # Apply thresholds
    promote_log = (act >= theta_imp) or (imp >= theta_imp) or novel
    promote_theory = (rec >= theta_rec) and (form >= theta_theory)

    # Promote to Logs
    if promote_log:
        log_file = log_folder / ram_file.name.replace("fragment", "log")
        shutil.copy(ram_file, log_file)
        print(f"Promoted to Log: {ram_file.name}")

    # Promote to Theory
    if promote_theory:
        theory_file = theory_folder / ram_file.name.replace("fragment", "theory")
        shutil.copy(ram_file, theory_file)
        print(f"Promoted to Theory: {ram_file.name}")

# ----------------------------
# Update extractor thresholds (mutation)
# ----------------------------
extractor["θ_rec"] = mutate_threshold(theta_rec)
extractor["θ_imp"] = mutate_threshold(theta_imp)
extractor["θ_theory"] = mutate_threshold(theta_theory)

# Save updated population
with open(population_file, "w") as f:
    json.dump(population, f, indent=2)

# ----------------------------
# Log cycle stats
# ----------------------------
import csv
import datetime

fieldnames = ["date","RAM","Logs","Theory","Novel"]
file_exists = history_file.exists()
with open(history_file, "a", newline="") as csvfile:
    writer = csv.DictWriter(csvfile, fieldnames=fieldnames)
    if not file_exists:
        writer.writeheader()
    writer.writerow({
        "date": datetime.date.today().isoformat(),
        "RAM": cycle_stats["RAM"],
        "Logs": cycle_stats["Logs"],
        "Theory": cycle_stats["Theory"],
        "Novel": cycle_stats["Novel"]
    })

print("Cycle complete. Extractor thresholds mutated and stats logged.")
