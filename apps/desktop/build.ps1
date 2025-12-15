# Voice Copilot Build Script
# Builds both MSI installer and portable executable

param(
    [switch]$Release,
    [switch]$Installer,
    [switch]$Portable,
    [switch]$All
)

$ErrorActionPreference = "Stop"

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "    Voice Copilot Build Script" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Default to All if no specific option
if (-not ($Release -or $Installer -or $Portable -or $All)) {
    $All = $true
}

if ($All) {
    $Release = $true
    $Installer = $true
    $Portable = $true
}

# Create output directory
$distDir = "dist"
if (-not (Test-Path $distDir)) {
    New-Item -ItemType Directory -Path $distDir | Out-Null
}

# Build release binary
if ($Release) {
    Write-Host "[1/3] Building release binary..." -ForegroundColor Yellow
    cargo build --release
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Build failed!" -ForegroundColor Red
        exit 1
    }
    Write-Host "Build complete!" -ForegroundColor Green
    Write-Host ""
}

# Create portable executable
if ($Portable) {
    Write-Host "[2/3] Creating portable executable..." -ForegroundColor Yellow

    $portableDir = "$distDir\voice-copilot-portable"
    if (Test-Path $portableDir) {
        Remove-Item -Recurse -Force $portableDir
    }
    New-Item -ItemType Directory -Path $portableDir | Out-Null

    # Copy executable
    Copy-Item "target\release\voice-copilot.exe" "$portableDir\"

    # Copy prompts
    if (Test-Path "prompts") {
        Copy-Item -Recurse "prompts" "$portableDir\"
    }

    # Create readme
    @"
Voice Copilot - Portable Edition
================================

Just run voice-copilot.exe to start!

First Run:
1. Click the audio source dropdown
2. Select your meeting app (Zoom, Discord, Teams, etc.)
3. Click "Start Listening"

API Keys:
- Create a .env file in this folder with your API keys
- Or enter them in Settings

For more info: https://github.com/HEKAX/voice-copilot
"@ | Out-File "$portableDir\README.txt" -Encoding UTF8

    # Create zip
    $zipPath = "$distDir\voice-copilot-portable.zip"
    if (Test-Path $zipPath) {
        Remove-Item $zipPath
    }
    Compress-Archive -Path "$portableDir\*" -DestinationPath $zipPath

    Write-Host "Portable version created: $zipPath" -ForegroundColor Green
    Write-Host ""
}

# Create MSI installer
if ($Installer) {
    Write-Host "[3/3] Creating MSI installer..." -ForegroundColor Yellow

    # Check if cargo-wix is installed
    $wixInstalled = cargo wix --version 2>$null
    if (-not $wixInstalled) {
        Write-Host "Installing cargo-wix..." -ForegroundColor Yellow
        cargo install cargo-wix
    }

    # Create assets directory if needed
    if (-not (Test-Path "assets")) {
        New-Item -ItemType Directory -Path "assets" | Out-Null
    }

    # Create a simple icon if not exists (placeholder)
    if (-not (Test-Path "assets\icon.ico")) {
        Write-Host "Note: No icon.ico found in assets folder. Using default." -ForegroundColor Yellow
        # The installer will use Windows default icon
    }

    # Build MSI
    cargo wix --no-build --nocapture
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Note: MSI creation requires WiX Toolset. Install from https://wixtoolset.org/" -ForegroundColor Yellow
        Write-Host "Skipping MSI creation." -ForegroundColor Yellow
    } else {
        # Move MSI to dist
        $msiFiles = Get-ChildItem "target\wix\*.msi"
        foreach ($msi in $msiFiles) {
            Move-Item $msi.FullName "$distDir\" -Force
        }
        Write-Host "MSI installer created in $distDir\" -ForegroundColor Green
    }
    Write-Host ""
}

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "    Build Complete!" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "Output files in: $distDir\" -ForegroundColor White
Get-ChildItem $distDir | ForEach-Object {
    Write-Host "  - $($_.Name)" -ForegroundColor Gray
}
Write-Host ""
