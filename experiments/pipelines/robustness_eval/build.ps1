# build.ps1
# PDF build with logging, numeric ordering, and SVG handling

# Set folders
$SrcDir = "src"
$OutDir = "output"
$LogDir = "logs"
$FigDir = "figures"

# Create directories if they don't exist
foreach ($dir in @($OutDir, $LogDir)) {
    if (!(Test-Path $dir)) { New-Item -ItemType Directory -Path $dir | Out-Null }
}

# Prepare log file
$timestamp = Get-Date -Format "yyyy-MM-dd_HH-mm-ss"
$logFile = Join-Path $LogDir "build_$timestamp.log"
Add-Content $logFile "PDF Build started at $(Get-Date)"

# Optional: Convert SVG figures to PDF for LaTeX
Get-ChildItem -Path $FigDir -Filter *.svg | ForEach-Object {
    $svgFile = $_.FullName
    $pdfFile = Join-Path $FigDir ($_.BaseName + ".pdf")
    if (!(Test-Path $pdfFile)) {
        # Requires Inkscape installed and in PATH
        inkscape $svgFile --export-type=pdf --export-filename=$pdfFile
        $entry = "[{0}] Converted SVG to PDF: {1}" -f (Get-Date -Format "yyyy-MM-dd HH:mm:ss"), $pdfFile
        Add-Content $logFile $entry
        Write-Host $entry
    }
}

# Process Markdown files in numeric order
Get-ChildItem -Path $SrcDir -Filter *.md | Sort-Object Name | ForEach-Object {
    $mdFile = $_.FullName
    $pdfFile = Join-Path $OutDir ($_.BaseName + ".pdf")

    # Build PDF via Pandoc + XeLaTeX
    pandoc $mdFile `
        --pdf-engine=xelatex `
        --template=template.tex `
        --bibliography=references.bib `
        --citeproc `
        --number-sections `
        -o $pdfFile

    # Log success
    $entry = "[{0}] Built PDF: {1}" -f (Get-Date -Format "yyyy-MM-dd HH:mm:ss"), $pdfFile
    Add-Content $logFile $entry
    Write-Host $entry
}

Add-Content $logFile "PDF Build finished at $(Get-Date)"
Write-Host "All Markdown files processed. Log saved to $logFile"

# =======================================
# Grove Snapshot Lock (Clean Execution)
# =======================================

# ---------------------------------------
# Resolve Script Directory
# ---------------------------------------
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path

# ---------------------------------------
# Get Git Repo Root (robust)
# ---------------------------------------
$ProjectRoot = git rev-parse --show-toplevel 2>$null
if (-not $ProjectRoot) {
    Write-Error "Not inside a Git repository. Exiting."
    exit 1
}
Set-Location $ProjectRoot

# ---------------------------------------
# Enforce Clean Repo
# ---------------------------------------
$Dirty = git status --porcelain
if ($Dirty) {
    Write-Host "Your working directory is dirty. Please commit or stash changes before running a snapshot:"
    git status --short
    exit 1
}

# ---------------------------------------
# Create Timestamped Snapshot Folder
# ---------------------------------------
$Timestamp   = Get-Date -Format "yyyyMMdd-HHmmss"
$RunPath     = Join-Path $ScriptDir "results\$Timestamp"
$CapsulePath = Join-Path $RunPath "capsule"
$OutputsPath = Join-Path $RunPath "outputs"

# Ensure directories exist
New-Item -ItemType Directory -Path $CapsulePath -Force | Out-Null
New-Item -ItemType Directory -Path $OutputsPath -Force | Out-Null

# ---------------------------------------
# Capture Git Provenance
# ---------------------------------------
git rev-parse HEAD | Out-File (Join-Path $CapsulePath "commit.txt") -Encoding utf8
git branch --show-current | Out-File (Join-Path $CapsulePath "branch.txt") -Encoding utf8
git status --porcelain | Out-File (Join-Path $CapsulePath "dirty_state.txt") -Encoding utf8

# ---------------------------------------
# Capture Python Environment
# ---------------------------------------
pip freeze | Out-File (Join-Path $CapsulePath "environment.lock.txt") -Encoding utf8

# ---------------------------------------
# Copy Pipeline Config
# ---------------------------------------
$PipelineConfig = Join-Path $ScriptDir "pipeline.yaml"
if (Test-Path $PipelineConfig) {
    Copy-Item $PipelineConfig $CapsulePath
} else {
    Write-Host "Warning: pipeline.yaml not found, skipping copy."
}

# ---------------------------------------
# Run Pipeline
# ---------------------------------------
$RunLog = Join-Path $RunPath "run.log"

Write-Host "Running pipeline..."
python "$ScriptDir/src/run.py" --output $OutputsPath 2>&1 | Tee-Object $RunLog

Write-Host "Snapshot created at: $RunPath"

# ---------------------------------------
# Generate Cryptographic Manifest
# ---------------------------------------

Write-Host "Generating snapshot manifest..."

$ManifestPath = Join-Path $CapsulePath "manifest.json"

# Collect commit again for redundancy
$CommitHash = git rev-parse HEAD

# Gather all output files
$OutputFiles = Get-ChildItem -Path $OutputsPath -Recurse -File

$FileHashes = @()

foreach ($File in $OutputFiles) {
    $Hash = Get-FileHash -Path $File.FullName -Algorithm SHA256
    $RelativePath = $File.FullName.Substring($RunPath.Length + 1)

    $FileHashes += [PSCustomObject]@{
        file  = $RelativePath
        sha256 = $Hash.Hash
    }
}

# Build manifest object
$Manifest = [PSCustomObject]@{
    timestamp = $Timestamp
    commit    = $CommitHash
    file_count = $FileHashes.Count
    files     = $FileHashes
}

# Write manifest as JSON
$Manifest | ConvertTo-Json -Depth 5 | Out-File $ManifestPath -Encoding utf8

Write-Host "Manifest written to $ManifestPath"
