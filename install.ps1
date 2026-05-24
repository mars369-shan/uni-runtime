<#
.SYNOPSIS
Uni-Runtime Windows Installer

.DESCRIPTION
Automatically downloads and installs Uni-Runtime with all necessary dependencies

.NOTES
Requires PowerShell to be run as Administrator
#>

param(
    [string]$Version = "0.1.0",
    [string]$InstallDir = "$env:USERPROFILE\.uni-runtime"
)

$ErrorActionPreference = "Stop"

Write-Host "=== Uni-Runtime Windows Installer ===" -ForegroundColor Cyan
Write-Host "Version: $Version" -ForegroundColor Cyan
Write-Host ""

# Check PowerShell version
if ($PSVersionTable.PSVersion.Major -lt 5) {
    Write-Error "PowerShell 5.0 or higher is required"
    exit 1
}

# Check if running as Administrator
$currentPrincipal = New-Object Security.Principal.WindowsPrincipal([Security.Principal.WindowsIdentity]::GetCurrent())
$isAdmin = $currentPrincipal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)

if (-not $isAdmin) {
    Write-Warning "It is recommended to run as Administrator for better experience"
    Write-Host "Continuing with installation...`n" -ForegroundColor Yellow
}

# Detect system architecture
$arch = if ([Environment]::Is64BitOperatingSystem) { "amd64" } else { "386" }
Write-Host "Detected system: Windows-$arch" -ForegroundColor Green

# Check dependencies
Write-Host "`nChecking dependencies..." -ForegroundColor Cyan

# Check Chocolatey
if (-not (Get-Command choco -ErrorAction SilentlyContinue)) {
    Write-Host "Installing Chocolatey package manager..." -ForegroundColor Yellow
    Set-ExecutionPolicy Bypass -Scope Process -Force; [System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072; iex ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))
    
    # Refresh PATH
    $env:Path = [System.Environment]::GetEnvironmentVariable("Path","Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path","User")
}

# Install necessary dependencies
$dependencies = @("docker-cli", "wsl2")

foreach ($dep in $dependencies) {
    if (-not (Get-Command $dep -ErrorAction SilentlyContinue)) {
        Write-Host "Installing $dep..." -ForegroundColor Yellow
        choco install $dep -y
    } else {
        Write-Host "$dep is already installed" -ForegroundColor Green
    }
}

# Enable WSL
Write-Host "`nConfiguring WSL..." -ForegroundColor Cyan
if (-not (Get-WindowsOptionalFeature -Online -FeatureName Microsoft-Windows-Subsystem-Linux).State -eq "Enabled") {
    Write-Host "Enabling WSL feature..." -ForegroundColor Yellow
    dism.exe /online /enable-feature /featurename:Microsoft-Windows-Subsystem-Linux /all /norestart
}

if (-not (Get-WindowsOptionalFeature -Online -FeatureName VirtualMachinePlatform).State -eq "Enabled") {
    Write-Host "Enabling Virtual Machine Platform..." -ForegroundColor Yellow
    dism.exe /online /enable-feature /featurename:VirtualMachinePlatform /all /norestart
}

# Set WSL2 as default version
wsl --set-default-version 2

# Create installation directory
Write-Host "`nCreating installation directory..." -ForegroundColor Cyan
New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null

# Download binary
$binaryUrl = "https://github.com/yourusername/uni-runtime/releases/download/v$Version/uni-runtime-windows-$arch.exe"
$binaryPath = "$InstallDir\uni-runtime.exe"

Write-Host "Downloading binary..." -ForegroundColor Cyan
Write-Host "URL: $binaryUrl"

try {
    [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
    Invoke-WebRequest -Uri $binaryUrl -OutFile $binaryPath -UseBasicParsing
    Write-Host "Download completed" -ForegroundColor Green
} catch {
    Write-Error "Download failed: $_"
    exit 1
}

# Add to PATH
Write-Host "`nConfiguring environment variables..." -ForegroundColor Cyan
$userPath = [Environment]::GetEnvironmentVariable("Path", "User")

if (-not $userPath.Contains($InstallDir)) {
    Write-Host "Adding $InstallDir to user PATH..." -ForegroundColor Yellow
    $newPath = "$userPath;$InstallDir"
    [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
    $env:Path += ";$InstallDir"
    Write-Host "Added to PATH" -ForegroundColor Green
} else {
    Write-Host "$InstallDir is already in PATH" -ForegroundColor Green
}

# Create config directories
$configDir = "$env:USERPROFILE\.config\uni-runtime"
$envsDir = "$env:USERPROFILE\.local\share\uni-runtime\envs"
New-Item -ItemType Directory -Path $configDir -Force | Out-Null
New-Item -ItemType Directory -Path $envsDir -Force | Out-Null

Write-Host ""
Write-Host "=== Installation Complete ===" -ForegroundColor Green
Write-Host ""
Write-Host "Usage:" -ForegroundColor Cyan
Write-Host "  uni-runtime create <env-name> --distro <distro>"
Write-Host "  uni-runtime exec <env-name> -- <command>"
Write-Host "  uni-runtime list"
Write-Host ""
Write-Host "Supported distributions:" -ForegroundColor Cyan
Write-Host "  ubuntu-22.04, ubuntu-24.04, debian-12"
Write-Host "  fedora-39, fedora-40, rhel-9"
Write-Host "  archlinux, opensuse-tumbleweed, opensuse-leap"
Write-Host ""
Write-Host "Note: Make sure Docker Desktop is running before first use" -ForegroundColor Yellow
