import os
import re

def generate_architecture_tree():
    target_file = os.path.join("docs", "architecture.md")
    os.makedirs("docs", exist_ok=True)
    
    # 1. Exact Name Block: Fast initial cuts to prevent stepping into massive dependency nests
    ignore_names = {
        '.git', 'node_modules', 'bin', 'obj', 'venv', '.venv', '__pycache__',
        'logs', 'fingerprint', 'deps', 'incremental', 'results', 'target', 'tests'
    }
    
    # 2. Structural Regex Rules: Matches recursive path wildcards such as:
    # - experiments/**/results/
    # - experiments/**/logs/
    # - Any nested **/target/ variants
    path_regex_filters = [
        re.compile(r"experiments[\\/].*[\\/](results|logs)"),
        re.compile(r".*[\\/]target([\\/]|\$)")
    ]
    
    with open(target_file, "w", encoding="utf-8") as f:
        f.write("## Internal Architecture Overview\n")
        f.write("```text\n")
        
        root_dir = os.getcwd()
        
        for root, dirs, files in os.walk(root_dir):
            # Compute path relative to project root for structural regex evaluation
            relative_root = os.path.relpath(root, root_dir)
            
            # Filter directory list in-place using combined logic string matching
            filtered_dirs = []
            for d in dirs:
                # Build what the full relative path would look like for this subdirectory
                sub_rel_path = d if relative_root == "." else os.path.join(relative_root, d)
                
                # Omit if directory matches an exact name filter or any regex wildcard rules
                is_ignored = (d in ignore_names) or any(rx.search(sub_rel_path) for rx in path_regex_filters)
                
                if not is_ignored:
                    filtered_dirs.append(d)
                    
            dirs[:] = filtered_dirs
            
            # Compute structural indentation depth
            if relative_root == ".":
                depth = 0
            else:
                depth = len(relative_root.split(os.sep))
                
            indent = "    " * depth
            
            # Print directory name block (except root directory self-references)
            if relative_root != ".":
                f.write(f"{indent[:-4]}├── {os.path.basename(root)}/\n")
                
            # Print file names safely attached to this layout branch
            for file in files:
                f.write(f"{indent}└── {file}\n")
                
        f.write("```\n")
        
    print("System structural blueprint successfully written to docs/architecture.md instantly.")

if __name__ == "__main__":
    generate_architecture_tree()