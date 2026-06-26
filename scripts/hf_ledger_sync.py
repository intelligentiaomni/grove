import json
import os
from datetime import datetime

class ResearchLedger:
    def __init__(self, filepath="ledger.json"):
        self.filepath = filepath
        self.data = self.load_ledger()

    def load_ledger(self):
        if not os.path.exists(self.filepath):
            return {
                "research_question": "How can early stopping criteria be optimized to prevent overfitting in edge-computing environments with strict hardware and memory limits?",
                "sources": {"datasets": [], "literature": []}
            }
        with open(self.filepath, "r", encoding="utf-8") as f:
            return json.load(f)

    def register_hf_dataset_chunk(self, file_hash, repo, split, filename, byte_range, topics):
        """Logs zero-dependency parquet stream targets for the Rust engine."""
        entry = {
            "id": f"sha256_{file_hash}",
            "repo": repo,
            "split": split,
            "parquet_file": filename,
            "byte_range": byte_range,
            "extracted_topics": topics,
            "provenance_node_bound": True
        }
        # Prevent duplicates
        if not any(d["id"] == entry["id"] for d in self.data["sources"]["datasets"]):
            self.data["sources"]["datasets"].append(entry)
            self.save()

    def register_hf_paper(self, paper_id, title, authors, url, nodes):
        """Logs literature papers scraped or served via Hugging Face Papers."""
        entry = {
            "id": f"hf_paper_{paper_id}",
            "repo": "huggingface/papers",
            "title": title,
            "authors": authors,
            "doi_or_url": url,
            "extracted_nodes": nodes,
            "download_timestamp": datetime.utcnow().isoformat() + "Z"
        }
        if not any(p["id"] == entry["id"] for p in self.data["sources"]["literature"]):
            self.data["sources"]["literature"].append(entry)
            self.save()

    def save(self):
        with open(self.filepath, "w", encoding="utf-8") as f:
            json.dump(self.data, f, indent=2, ensure_ascii=False)

# --- Example Usage ---
ledger = ResearchLedger()

# Mocking ingestion registration from a FineWeb-Edu range request slice
ledger.register_hf_dataset_chunk(
    file_hash="9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08",
    repo="HuggingFaceFW/fineweb-edu",
    split="train",
    filename="data/001.parquet",
    byte_range="0-5000000",
    topics=["overfitting-mitigation", "tiny-ml"]
)
