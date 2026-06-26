import json
import os
import sys

def generate_mermaid_dag(ledger_path="ledger.json"):
    """Backwards compatibility wrapper pointing to the new visualization compiler."""
    return compile_ledger_to_mermaid(ledger_path)

def compile_ledger_to_mermaid(ledger_path="ledger.json"):
    """Reads a validated JSON ledger and maps complete source data provenance with styling."""
    if not os.path.exists(ledger_path):
        print(f"Error: Ledger file '{ledger_path}' not found.", file=sys.stderr)
        return None

    try:
        with open(ledger_path, "r", encoding="utf-8") as f:
            data = json.load(f)
    except Exception as e:
        print(f"Error reading or parsing JSON from '{ledger_path}': {e}", file=sys.stderr)
        return None

    # Base graph initialization (Top-Down layout)
    lines = [
        "```mermaid",
        "graph TD",
        "  %% Styling Rules",
        "  classDef question fill:#f9f,stroke:#333,stroke-width:2px,font-weight:bold;",
        "  classDef dataset fill:#bbf,stroke:#333,stroke-width:1px;",
        "  classDef paper fill:#bfb,stroke:#333,stroke-width:1px;",
        "  classDef topic fill:#fbb,stroke:#333,stroke-width:1px,font-style:italic;",
        ""
    ]

    # 1. Define Root Node
    raw_q = data.get("research_question", "Machine Learning Task")
    # Truncate string for node presentation size
    truncated_q = raw_q[:50] + "..." if len(raw_q) > 50 else raw_q
    lines.append(f'  Q["❓ {truncated_q}"]:::question')
    lines.append("")

    sources = data.get("sources", {})

    # 2. Process Streaming Datasets
    lines.append("  %% Dataset Provenance Branches")
    for ds in sources.get("datasets", []):
        full_id = ds["id"]
        # Use short-hash for clean node IDs (safely handling trailing string parsing)
        short_hash = full_id.split("_")[-1][:8] if "_" in full_id else full_id[-8:]
        node_id = f"DS_{short_hash}"
        
        file_name = ds["parquet_file"].split("/")[-1]
        lines.append(f'  {node_id}["📦 {ds["repo"]}/{file_name}"]:::dataset')
        lines.append(f"  Q --> {node_id}")

        # Connect extracted topic flags
        for topic in ds.get("extracted_topics", []):
            topic_clean = topic.replace("-", "_").replace(" ", "_")
            lines.append(f'  T_{topic_clean}["🧬 {topic}"]:::topic')
            lines.append(f"  {node_id} --> T_{topic_clean}")
    lines.append("")

    # 3. Process Academic Literature
    lines.append("  %% Literature Evidence Branches")
    for lit in sources.get("literature", []):
        full_id = lit["id"]
        # Replace decimals/dashes with underscores to respect Mermaid identifier limits
        lit_id = "PAP_" + full_id.replace(".", "_").replace("-", "_").replace(" ", "_")
        
        title_trunc = lit["title"][:40] + "..." if len(lit["title"]) > 40 else lit["title"]
        lines.append(f'  {lit_id}["📄 {title_trunc}"]:::paper')
        lines.append(f"  Q --> {lit_id}")

        # Connect extracted node flags
        for node in lit.get("extracted_nodes", []):
            node_clean = node.replace("-", "_").replace(" ", "_")
            lines.append(f'  T_{node_clean}["🧬 {node}"]:::topic')
            lines.append(f"  {lit_id} --> T_{node_clean}")

    lines.append("```")
    return "\n".join(lines)


if __name__ == "__main__":
    mermaid_markdown = compile_ledger_to_mermaid()
    if mermaid_markdown:
        print("\n=== GENERATED PROVENANCE DAG ===")
        print(mermaid_markdown)
        
        # Auto-save to a local documentation page
        try:
            with open("PROVENANCE.md", "w", encoding="utf-8") as out:
                out.write("# Research Data Lineage\n\n")
                out.write(mermaid_markdown)
            print("=================================")
            print("Saved markdown map directly to PROVENANCE.md")
        except Exception as e:
            print(f"Error saving to PROVENANCE.md: {e}", file=sys.stderr)
