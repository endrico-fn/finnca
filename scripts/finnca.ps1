# ==============================================================================
# Finnca — Industrial Finance Ledger Workstation (Windows Automation Engine)
# Usage:
#   irm https://raw.githubusercontent.com/endrico-fn/finnca/main/scripts/finnca.ps1 | iex
#   powershell -ExecutionPolicy Bypass -File scripts\finnca.ps1 [install|upgrade|uninstall|status]
# ==============================================================================

[CmdletBinding()]
param (
    [Parameter(Position = 0)]
    [ValidateSet("install", "upgrade", "uninstall", "status")]
    [string]$Action = "install",

    [Parameter(Position = 1)]
    [string]$TargetVersion = "",

    [Parameter()]
    [switch]$Local
)

$ErrorActionPreference = "Stop"

$Repo = "endrico-fn/finnca"
$AppName = "Finnca"
$AppSlug = "finnca"

# Utilitarian Industrial Terminal Formatting
function Write-Header {
    Write-Host ""
    Write-Host "=================================================================" -ForegroundColor DarkGray
    Write-Host "  FINNCA // WINDOWS DEPLOYMENT & WORKSTATION ENGINE" -ForegroundColor Cyan
    Write-Host "=================================================================" -ForegroundColor DarkGray
    Write-Host ""
}

function Write-Step {
    param ([string]$Message)
    Write-Host "  -> $Message" -ForegroundColor Gray
}

function Write-Success {
    param ([string]$Message)
    Write-Host "  [OK] $Message" -ForegroundColor Green
}

function Write-Warn {
    param ([string]$Message)
    Write-Host "  [WARN] $Message" -ForegroundColor Yellow
}

function Write-Fail {
    param ([string]$Message)
    Write-Host "  [ERR] $Message" -ForegroundColor Red
}

function Get-LatestReleaseInfo {
    param ([string]$Version = "")
    $Url = if ($Version) {
        "https://api.github.com/repos/$Repo/releases/tags/$Version"
    } else {
        "https://api.github.com/repos/$Repo/releases/latest"
    }

    try {
        [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
        $Release = Invoke-RestMethod -Uri $Url -Headers @{ "User-Agent" = "finnca-windows-installer" }
        return $Release
    } catch {
        Write-Fail "Unable to query release metadata from GitHub."
        Write-Step "Endpoint: $Url"
        Write-Step "Detail: $($_.Exception.Message)"
        throw $_
    }
}

function Install-Finnca {
    param ([switch]$UseLocal)

    $LocalInstaller = $null
    $ProjectRoot = if ($PSScriptRoot) { Split-Path -Parent $PSScriptRoot } else { Get-Location }
    $LocalBundleDir = Join-Path $ProjectRoot "src-tauri\target\release\bundle\nsis"
    if (Test-Path $LocalBundleDir) {
        $LocalFile = Get-ChildItem -Path $LocalBundleDir -Filter "*setup.exe" -ErrorAction SilentlyContinue | Select-Object -First 1
        if ($LocalFile) {
            $LocalInstaller = $LocalFile.FullName
        }
    }

    if (($UseLocal -or $Local -or $TargetVersion -eq "local") -and $LocalInstaller) {
        Write-Step "Using local build artifact: $LocalInstaller"
        $TempFile = $LocalInstaller
        $InstallerName = Split-Path -Leaf $LocalInstaller
        $Tag = "local"
    } else {
        Write-Step "Resolving latest production release artifact..."
        $Release = Get-LatestReleaseInfo -Version $TargetVersion
        $Tag = $Release.tag_name
        Write-Step "Identified release: $Tag"

        # Look for NSIS per-user setup executable (*x64*setup.exe or *setup.exe, excluding .sig)
        $Asset = $Release.assets | Where-Object { ($_.name -like "*x64*setup.exe" -or $_.name -like "*setup.exe") -and $_.name -notlike "*.sig" } | Select-Object -First 1
        if (-not $Asset) {
            # Fallback to any .exe or legacy .msi if setup.exe is not found
            $Asset = $Release.assets | Where-Object { ($_.name -like "*.exe" -or $_.name -like "*.msi") -and $_.name -notlike "*.sig" } | Select-Object -First 1
        }

        if (-not $Asset) {
            Write-Fail "No compatible Windows NSIS setup binary (*-setup.exe) found in release $Tag."
            return
        }

        $InstallerName = $Asset.name
        $DownloadUrl = $Asset.browser_download_url
        $TempDir = [System.IO.Path]::GetTempPath()
        $TempFile = Join-Path $TempDir $InstallerName

        Write-Step "Downloading: $InstallerName ($([math]::Round($Asset.size / 1MB, 1)) MB)..."
        try {
            Invoke-WebRequest -Uri $DownloadUrl -OutFile $TempFile -UseBasicParsing
        } catch {
            Write-Fail "Download failed: $($_.Exception.Message)"
            return
        }
    }

    Write-Step "Deploying package..."
    if ($InstallerName -like "*.msi") {
        Write-Step "Running Windows Installer (passive mode)..."
        $Process = Start-Process msiexec.exe -ArgumentList "/i `"$TempFile`" /passive /norestart" -Wait -PassThru
        if ($Process.ExitCode -ne 0 -and $Process.ExitCode -ne 3010) {
            Write-Fail "Installation failed with exit code: $($Process.ExitCode)"
            return
        }
    } else {
        Write-Step "Running NSIS setup executable..."
        $Process = Start-Process -FilePath $TempFile -ArgumentList "/S" -Wait -PassThru
        if ($Process.ExitCode -ne 0) {
            Write-Fail "Installation failed with exit code: $($Process.ExitCode)"
            return
        }
    }

    # Clean up temp installer if downloaded from remote
    if (Test-Path $TempFile -and $TempFile -ne $LocalInstaller) {
        Remove-Item -Force $TempFile -ErrorAction SilentlyContinue
    }

    Write-Success "Finnca ($Tag) successfully deployed on this workstation."
    Write-Host ""
    Write-Host "  -> Desktop & Start Menu shortcuts are ready." -ForegroundColor Gray
    Write-Host "  -> Vault storage invariant: %APPDATA%\finnca" -ForegroundColor Gray
    Write-Host ""
}

function Show-Status {
    Write-Step "Inspecting Finnca installation on Windows..."

    $ConfigDir = Join-Path $env:APPDATA "finnca"
    $ConfigFile = Join-Path $ConfigDir "finnca.json"

    Write-Host ""
    Write-Host "  [Storage Invariants]" -ForegroundColor Cyan
    Write-Host "    Configuration Directory : $ConfigDir" -ForegroundColor Gray
    if (Test-Path $ConfigFile) {
        Write-Host "    Configuration File      : PRESENT" -ForegroundColor Green
    } else {
        Write-Host "    Configuration File      : NONE (First run or pristine)" -ForegroundColor Yellow
    }

    # Detect common installation locations
    $PossiblePaths = @(
        (Join-Path $env:LOCALAPPDATA "Programs\finnca\finnca.exe"),
        "${env:ProgramFiles}\finnca\finnca.exe",
        "${env:ProgramFiles(x86)}\finnca\finnca.exe"
    )

    $FoundPath = $PossiblePaths | Where-Object { Test-Path $_ } | Select-Object -First 1

    Write-Host ""
    Write-Host "  [Binary Verification]" -ForegroundColor Cyan
    if ($FoundPath) {
        $VersionInfo = (Get-Item $FoundPath).VersionInfo
        Write-Host "    Binary Location : $FoundPath" -ForegroundColor Green
        Write-Host "    Product Version : $($VersionInfo.ProductVersion)" -ForegroundColor Green
    } else {
        Write-Host "    Binary Location : NOT FOUND in standard installation paths" -ForegroundColor Yellow
    }
    Write-Host ""
}

function Uninstall-Finnca {
    Write-Step "Locating installed uninstaller..."

    # Common uninstaller paths
    $PossibleUninstallers = @(
        (Join-Path $env:LOCALAPPDATA "Programs\finnca\Uninstall finnca.exe"),
        "${env:ProgramFiles}\finnca\Uninstall finnca.exe"
    )

    $Uninstaller = $PossibleUninstallers | Where-Object { Test-Path $_ } | Select-Object -First 1

    if ($Uninstaller) {
        Write-Step "Executing uninstaller: $Uninstaller"
        Start-Process -FilePath $Uninstaller -ArgumentList "/S" -Wait
        Write-Success "Finnca uninstalled cleanly."
    } else {
        # Check Windows Registry Uninstall key for MSI
        Write-Step "Searching Windows Registry uninstall catalog..."
        $RegKey = "HKLM:\Software\Microsoft\Windows\CurrentVersion\Uninstall\*"
        $App = Get-ItemProperty $RegKey -ErrorAction SilentlyContinue | Where-Object { $_.DisplayName -eq "finnca" -or $_.DisplayName -eq "Finnca" }
        if ($App -and $App.UninstallString) {
            Write-Step "Running: $($App.UninstallString)"
            Start-Process cmd.exe -ArgumentList "/c $($App.UninstallString) /quiet /norestart" -Wait
            Write-Success "Finnca uninstalled cleanly via Windows Installer."
        } else {
            Write-Warn "No automated uninstaller found. You can uninstall via Windows Settings -> Apps."
        }
    }

    Write-Step "Safety invariant preserved: Vault files in %APPDATA%\finnca remain intact."
}

# --- Entry Point ---
Write-Header

switch ($Action.ToLower()) {
    "install"   { Install-Finnca -UseLocal:$Local }
    "upgrade"   { Install-Finnca -UseLocal:$Local }
    "uninstall" { Uninstall-Finnca }
    "status"    { Show-Status }
    default     { Write-Fail "Unknown action: $Action" }
}
