# ========================================
# build.ps1 — Grove PDF + Snapshot Lock
# Incremental-safe PDF + Force Rebuild + Old Results Cleanup
# Safe Move-Item for snapshots
# ========================================

param(
    [switch]$ForceRebuild  # Use -ForceRebuild to regenerate all PDFs and SVGs
)

# ---------------------------------------
# Resolve Script Directory & Git Root
# ---------------------------------------
$ScriptDir   = Split-Path -Parent $MyInvocation.MyCommand.Path
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
# Configuration
# ---------------------------------------
$MaxSnapshots = 5
$ResultsDir   = Join-Path $ProjectRoot "results"

# ---------------------------------------
# Cleanup old snapshots
# ---------------------------------------
if (Test-Path $ResultsDir) {
    $oldSnapshots = Get-ChildItem -Path $ResultsDir -Directory | Sort-Object Name -Descending
    if ($oldSnapshots.Count -gt $MaxSnapshots) {
        $toRemove = $oldSnapshots[$MaxSnapshots..($oldSnapshots.Count - 1)]
        foreach ($folder in $toRemove) {
            Write-Host "Removing old snapshot folder: $($folder.FullName)"
            Remove-Item -Recurse -Force $folder.FullName
        }
    }
}

# ---------------------------------------
# Set Paths
# ---------------------------------------
$PipelineSrc   = Join-Path $ProjectRoot "experiments/pipelines/robustness_eval/src"
$TempRunPath   = Join-Path $ScriptDir "results_temp"
$OutputsTemp   = Join-Path $TempRunPath "outputs"
$CapsuleTemp   = Join-Path $TempRunPath "capsule"
$FigDir        = Join-Path $PipelineSrc "figures"

$OutDir  = Join-Path $OutputsTemp "pdf"
$LogDir  = Join-Path $OutputsTemp "logs"

# ---------------------------------------
# Clean previous temp folder if exists
# ---------------------------------------
if (Test-Path $TempRunPath) {
    Write-Host "Cleaning previous temporary folder: $TempRunPath"
    Remove-Item -Recurse -Force $TempRunPath
}

# Ensure directories exist
foreach ($dir in @($OutDir, $LogDir, $CapsuleTemp)) {
    New-Item -ItemType Directory -Path $dir -Force | Out-Null
}

# ---------------------------------------
# Prepare log file
# ---------------------------------------
$timestamp = Get-Date -Format "yyyyMMdd-HHmmss"
$logFile   = Join-Path $LogDir "build_$timestamp.log"
Add-Content $logFile "PDF Build started at $(Get-Date)"
Write-Host "PDF Build started at $(Get-Date)"

# ---------------------------------------
# Convert SVG figures to PDF (incremental or forced)
# ---------------------------------------
if (Test-Path $FigDir) {
    Get-ChildItem -Path $FigDir -Filter *.svg | ForEach-Object {
        $svgFile = $_.FullName
        $pdfFile = Join-Path $FigDir ($_.BaseName + ".pdf")

        if ($ForceRebuild -or !(Test-Path $pdfFile)) {
            inkscape $svgFile --export-type=pdf --export-filename=$pdfFile
            $entry = "[{0}] Converted SVG to PDF: {1}" -f (Get-Date -Format "yyyy-MM-dd HH:mm:ss"), $pdfFile
        } else {
            $entry = "[{0}] Skipped existing PDF: {1}" -f (Get-Date -Format "yyyy-MM-dd HH:mm:ss"), $pdfFile
        }
        Add-Content $logFile $entry
        Write-Host $entry
    }
}

# ---------------------------------------
# Build PDFs from Markdown (incremental or forced)
# ---------------------------------------
Get-ChildItem -Path $PipelineSrc -Filter *.md | Sort-Object Name | ForEach-Object {
    $mdFile  = $_.FullName
    $pdfFile = Join-Path $OutDir ($_.BaseName + ".pdf")

    if ($ForceRebuild -or !(Test-Path $pdfFile)) {
        pandoc $mdFile `
            --pdf-engine=xelatex `
            --template=template.tex `
            --bibliography=references.bib `
            --citeproc `
            --number-sections `
            -o $pdfFile

        $entry = "[{0}] Built PDF: {1}" -f (Get-Date -Format "yyyy-MM-dd HH:mm:ss"), $pdfFile
    } else {
        $entry = "[{0}] Skipped existing PDF: {1}" -f (Get-Date -Format "yyyy-MM-dd HH:mm:ss"), $pdfFile
    }
    Add-Content $logFile $entry
    Write-Host $entry
}

Add-Content $logFile "PDF Build finished at $(Get-Date)"
Write-Host "All Markdown files processed. Log saved to $logFile"

# ---------------------------------------
# Run pipeline.py
# ---------------------------------------
$RunLog = Join-Path $LogDir "pipeline_$timestamp.log"

if (-not (Test-Path (Join-Path $PipelineSrc "run.py"))) {
    Write-Error "Pipeline run.py not found at $PipelineSrc. Cannot run pipeline."
    exit 1
}

Write-Host "Running pipeline..."
python (Join-Path $PipelineSrc "run.py") --output $OutputsTemp 2>&1 | Tee-Object $RunLog

# ---------------------------------------
# Create Timestamped Snapshot Folder
# ---------------------------------------
$RunPath     = Join-Path $ProjectRoot ("results\$timestamp")
$CapsulePath = Join-Path $RunPath "capsule"
$OutputsPath = Join-Path $RunPath "outputs"

# Ensure directories exist
New-Item -ItemType Directory -Path $RunPath -Force | Out-Null
New-Item -ItemType Directory -Path $CapsulePath -Force | Out-Null
New-Item -ItemType Directory -Path $OutputsPath -Force | Out-Null

# ---------------------------------------
# Move temporary outputs into final snapshot folder safely
# ---------------------------------------
Get-ChildItem -Path $TempRunPath -Force | ForEach-Object {
    $dest = Join-Path $RunPath $_.Name
    if (Test-Path $dest) {
        Remove-Item -Recurse -Force $dest
    }
    Move-Item $_.FullName $RunPath
}

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
$PipelineConfig = Join-Path $ProjectRoot "experiments/pipelines/robustness_eval/pipeline.yaml"
if (Test-Path $PipelineConfig) {
    Copy-Item $PipelineConfig $CapsulePath
} else {
    Write-Host "Warning: pipeline.yaml not found, skipping copy."
}

# ---------------------------------------
# Generate Cryptographic Manifest
# ---------------------------------------
Write-Host "Generating snapshot manifest..."

$ManifestPath = Join-Path $CapsulePath "manifest.json"
$CommitHash   = git rev-parse HEAD
$OutputFiles  = Get-ChildItem -Path $OutputsPath -Recurse -File

$FileHashes = @()
foreach ($File in $OutputFiles) {
    $Hash = Get-FileHash -Path $File.FullName -Algorithm SHA256
    $RelativePath = $File.FullName.Substring($RunPath.Length + 1)
    $FileHashes += [PSCustomObject]@{
        file   = $RelativePath
        sha256 = $Hash.Hash
    }
}

$Manifest = [PSCustomObject]@{
    timestamp  = $timestamp
    commit     = $CommitHash
    file_count = $FileHashes.Count
    files      = $FileHashes
}

$Manifest | ConvertTo-Json -Depth 5 | Out-File $ManifestPath -Encoding utf8
Write-Host "Manifest written to $ManifestPath"
Write-Host "Snapshot created at: $RunPath"
