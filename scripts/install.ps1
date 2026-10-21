# Galerne installer for Windows (PowerShell)
# Usage: irm https://raw.githubusercontent.com/Voldigoade/galerne/main/scripts/install.ps1 | iex

$ErrorActionPreference = "Stop"

$Repo = "Voldigoade/galerne"
$InstallDir = "$env:LOCALAPPDATA\Programs\galerne"
$ExePath = "$InstallDir\galerne.exe"

Write-Host "=== Galerne Windows Installer ===" -ForegroundColor Cyan

# Ensure target directory exists
if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
}

# Check if local release binary exists
if (Test-Path ".\target\release\galerne.exe") {
    Write-Host "Installing from local release build..." -ForegroundColor Green
    Copy-Item ".\target\release\galerne.exe" -Destination $ExePath -Force
} else {
    Write-Host "Downloading latest release package from GitHub..." -ForegroundColor Yellow
    $ZipUrl = "https://github.com/$Repo/releases/latest/download/galerne-x86_64-pc-windows-msvc.zip"
    $TempZip = "$env:TEMP\galerne.zip"

    try {
        Invoke-WebRequest -Uri $ZipUrl -OutFile $TempZip -UseBasicParsing
        Expand-Archive -Path $TempZip -DestinationPath $InstallDir -Force
        Remove-Item $TempZip -Force
    } catch {
        if (Get-Command cargo -ErrorAction SilentlyContinue) {
            Write-Host "Release asset not available yet; compiling with cargo..." -ForegroundColor Yellow
            cargo install --git "https://github.com/$Repo.git" --bin galerne
            return
        } else {
            throw "Unable to download Galerne release asset: $_"
        }
    }
}

# Add to User PATH if missing
$UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($UserPath -notlike "*$InstallDir*") {
    Write-Host "Adding $InstallDir to User PATH..." -ForegroundColor Green
    [Environment]::SetEnvironmentVariable("Path", "$UserPath;$InstallDir", "User")
    $env:Path += ";$InstallDir"
}

# Run setup to configure shell integration
if (Test-Path $ExePath) {
    & $ExePath setup
} else {
    Write-Host "Galerne installed to $InstallDir. Open a new terminal to use." -ForegroundColor Green
}
