# Voice Copilot Installer Builder
# Creates both MSI installer and portable ZIP

param(
    [switch]$SkipBuild,
    [switch]$PortableOnly,
    [switch]$MsiOnly
)

$ErrorActionPreference = "Stop"
$ProjectRoot = Split-Path -Parent $PSScriptRoot
$Version = "0.1.0"
$AppName = "VoiceCopilot"

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Voice Copilot Installer Builder" -ForegroundColor Cyan
Write-Host "  Version: $Version" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Step 1: Build Release
if (-not $SkipBuild) {
    Write-Host "[1/5] Building release binary..." -ForegroundColor Yellow
    Push-Location $ProjectRoot

    cargo build --release
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Build failed!" -ForegroundColor Red
        exit 1
    }

    Pop-Location
    Write-Host "Build complete!" -ForegroundColor Green
} else {
    Write-Host "[1/5] Skipping build (using existing binary)" -ForegroundColor Gray
}

# Step 2: Create output directory
$OutputDir = Join-Path $ProjectRoot "dist"
if (Test-Path $OutputDir) {
    Remove-Item -Recurse -Force $OutputDir
}
New-Item -ItemType Directory -Path $OutputDir | Out-Null
Write-Host "[2/5] Created output directory: $OutputDir" -ForegroundColor Green

# Step 3: Create portable ZIP
if (-not $MsiOnly) {
    Write-Host "[3/5] Creating portable ZIP..." -ForegroundColor Yellow

    $PortableDir = Join-Path $OutputDir "portable"
    New-Item -ItemType Directory -Path $PortableDir | Out-Null

    # Copy executable
    Copy-Item (Join-Path $ProjectRoot "target\release\voice-copilot.exe") $PortableDir

    # Create default .env template
    @"
# Voice Copilot Configuration
# Rename this file to .env and add your API keys

# Speech-to-Text (pick one)
DEEPGRAM_API_KEY=your_key_here
# OPENAI_API_KEY=your_key_here

# AI Models (optional - can use OpenAI for all)
# ANTHROPIC_API_KEY=your_key_here
# GOOGLE_AI_API_KEY=your_key_here
"@ | Out-File -FilePath (Join-Path $PortableDir ".env.template") -Encoding UTF8

    # Create README
    @"
Voice Copilot - Portable Edition
================================

Quick Start:
1. Rename .env.template to .env
2. Add your API keys to .env
3. Double-click voice-copilot.exe

Keyboard Shortcuts:
- Ctrl+Shift+S: Start/Stop listening
- Ctrl+Shift+H: Hide/Show window
- Ctrl+Shift+M: Switch mode
- Ctrl+Shift+C: Copy suggestion
- F8: Stealth mode (hide from everything)

For more info: https://github.com/AhmediHarhash/voice-copilot
"@ | Out-File -FilePath (Join-Path $PortableDir "README.txt") -Encoding UTF8

    # Create ZIP
    $ZipPath = Join-Path $OutputDir "$AppName-$Version-portable.zip"
    Compress-Archive -Path "$PortableDir\*" -DestinationPath $ZipPath -Force

    Write-Host "Portable ZIP created: $ZipPath" -ForegroundColor Green
}

# Step 4: Create MSI installer (requires WiX Toolset)
if (-not $PortableOnly) {
    Write-Host "[4/5] Creating MSI installer..." -ForegroundColor Yellow

    $WixDir = Join-Path $ProjectRoot "installer\wix"

    # Check if WiX is available
    $WixPath = Get-Command candle.exe -ErrorAction SilentlyContinue
    if (-not $WixPath) {
        # Try common install locations
        $WixLocations = @(
            "C:\Program Files (x86)\WiX Toolset v3.11\bin",
            "C:\Program Files\WiX Toolset v3.11\bin",
            "$env:USERPROFILE\.wix\bin"
        )

        foreach ($loc in $WixLocations) {
            if (Test-Path (Join-Path $loc "candle.exe")) {
                $env:PATH += ";$loc"
                break
            }
        }
    }

    # Check again
    $WixPath = Get-Command candle.exe -ErrorAction SilentlyContinue
    if (-not $WixPath) {
        Write-Host "WiX Toolset not found. Skipping MSI creation." -ForegroundColor Yellow
        Write-Host "Install WiX from: https://wixtoolset.org/releases/" -ForegroundColor Gray
    } else {
        # Create WiX source if not exists
        if (-not (Test-Path $WixDir)) {
            New-Item -ItemType Directory -Path $WixDir | Out-Null
        }

        # Generate WiX XML
        $WixSource = @"
<?xml version="1.0" encoding="UTF-8"?>
<Wix xmlns="http://schemas.microsoft.com/wix/2006/wi">
    <Product Id="*"
             Name="Voice Copilot"
             Language="1033"
             Version="$Version.0"
             Manufacturer="AhmediHarhash"
             UpgradeCode="A1B2C3D4-E5F6-7890-ABCD-EF1234567890">

        <Package InstallerVersion="200" Compressed="yes" InstallScope="perUser" />
        <MajorUpgrade DowngradeErrorMessage="A newer version is already installed." />
        <MediaTemplate EmbedCab="yes" />

        <Feature Id="ProductFeature" Title="Voice Copilot" Level="1">
            <ComponentGroupRef Id="ProductComponents" />
            <ComponentRef Id="ApplicationShortcut" />
            <ComponentRef Id="StartMenuShortcut" />
        </Feature>

        <Directory Id="TARGETDIR" Name="SourceDir">
            <Directory Id="LocalAppDataFolder">
                <Directory Id="INSTALLFOLDER" Name="VoiceCopilot">
                    <Component Id="ProductComponents" Guid="*">
                        <File Id="MainExe" Source="..\..\target\release\voice-copilot.exe" KeyPath="yes" />
                    </Component>
                </Directory>
            </Directory>

            <Directory Id="DesktopFolder" Name="Desktop">
                <Component Id="ApplicationShortcut" Guid="*">
                    <Shortcut Id="DesktopShortcut"
                              Name="Voice Copilot"
                              Target="[INSTALLFOLDER]voice-copilot.exe"
                              WorkingDirectory="INSTALLFOLDER" />
                    <RemoveFolder Id="RemoveDesktopFolder" On="uninstall" />
                    <RegistryValue Root="HKCU" Key="Software\VoiceCopilot" Name="DesktopShortcut" Type="integer" Value="1" KeyPath="yes" />
                </Component>
            </Directory>

            <Directory Id="ProgramMenuFolder">
                <Directory Id="ApplicationProgramsFolder" Name="Voice Copilot">
                    <Component Id="StartMenuShortcut" Guid="*">
                        <Shortcut Id="StartMenuShortcut"
                                  Name="Voice Copilot"
                                  Target="[INSTALLFOLDER]voice-copilot.exe"
                                  WorkingDirectory="INSTALLFOLDER" />
                        <RemoveFolder Id="RemoveStartMenuFolder" On="uninstall" />
                        <RegistryValue Root="HKCU" Key="Software\VoiceCopilot" Name="StartMenuShortcut" Type="integer" Value="1" KeyPath="yes" />
                    </Component>
                </Directory>
            </Directory>
        </Directory>

        <Icon Id="AppIcon" SourceFile="..\..\assets\icon.ico" />
        <Property Id="ARPPRODUCTICON" Value="AppIcon" />

    </Product>
</Wix>
"@

        $WixSourcePath = Join-Path $WixDir "product.wxs"
        $WixSource | Out-File -FilePath $WixSourcePath -Encoding UTF8

        # Compile and link
        Push-Location $WixDir

        & candle.exe product.wxs -o product.wixobj
        if ($LASTEXITCODE -eq 0) {
            & light.exe product.wixobj -o "$OutputDir\$AppName-$Version-setup.msi" -ext WixUIExtension
            if ($LASTEXITCODE -eq 0) {
                Write-Host "MSI installer created: $OutputDir\$AppName-$Version-setup.msi" -ForegroundColor Green
            }
        }

        Pop-Location
    }
}

# Step 5: Summary
Write-Host ""
Write-Host "[5/5] Build complete!" -ForegroundColor Green
Write-Host ""
Write-Host "Output files in: $OutputDir" -ForegroundColor Cyan

Get-ChildItem $OutputDir -File | ForEach-Object {
    $size = [math]::Round($_.Length / 1MB, 2)
    Write-Host "  - $($_.Name) ($size MB)" -ForegroundColor White
}

Write-Host ""
Write-Host "Done!" -ForegroundColor Green
