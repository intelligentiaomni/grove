import json
import os
from datetime import datetime

LEDGER_FILE = "ledger.json"

def load_ledger(filepath):
    """Safely loads the JSON ledger, initializing a blank one if missing."""
    if not os.path.exists(filepath):
        return {
            "research_question": "How can early stopping criteria be optimized to prevent overfitting in edge-computing environments with strict hardware and memory limits?",
            "search_queries": [],
            "corpus": []
        }
    with open(filepath, "r", encoding="utf-8") as f:
        return json.load(f)

def save_ledger(filepath, data):
    """Writes the updated Python dictionary back to a formatted JSON file."""
    with open(filepath, "w", encoding="utf-8") as f:
        # indent=4 ensures readability; ensure_ascii=False supports global characters
        json.dump(data, f, indent=4, ensure_ascii=False)

# --- Pipeline Execution ---
# 1. Load the ledger
ledger = load_ledger(LEDGER_FILE)

# 2. Append a new search query execution
new_search = {
    "query": "\"resource-constrained machine learning\" AND \"early stopping\"",
    "status": "pending",
    "timestamp": datetime.utcnow().isoformat() + "Z"
}
ledger["search_queries"].append(new_search)

# 3. Append a discovered paper to your corpus
new_paper = {
    "id": f"paper_{len(ledger['corpus']) + 1:03d}",
    "title": "Dynamic Patience Scaling under Edge Hardware Memory Saturation",
    "authors": ["Wang, L.", "Kim, Y."],
    "year": 2026,
    "abstract": "We present an optimization framework that scales validation patience down dynamically...",
    "relevance_score": 0.88
}
ledger["corpus"].append(new_paper)

# 4. Save updates
save_ledger(LEDGER_FILE, ledger)
print(f"Ledger successfully updated. Corpus size: {len(ledger['corpus'])}")