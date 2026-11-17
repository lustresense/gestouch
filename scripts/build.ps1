# Gestouch 0.0.1 — ONE-COMMAND Windows build (PowerShell 5.1+)
# Run from any directory: powershell -ExecutionPolicy Bypass -File .\scripts\build.ps1
# Optional: -Install; optional advanced: -Install -Service (elevated PowerShell).
[CmdletBinding()]
param([switch]$Install,[switch]$Service)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$root = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
Set-Location $root
function Step([string]$message) { Write-Host "`n==> $message" -ForegroundColor Cyan }
function Run([string]$exe,[string[]]$arguments) {
    & $exe @arguments
    if ($LASTEXITCODE -ne 0) { throw "$exe failed with exit code $LASTEXITCODE" }
}
Step 'Check Rust toolchain'
$cargo = Get-Command cargo -ErrorAction SilentlyContinue
if (-not $cargo) {
    $fallback = Join-Path $env:USERPROFILE '.cargo\bin\cargo.exe'
    if (Test-Path $fallback) { $cargo = Get-Item $fallback }
}
if (-not $cargo) {
    if (-not (Get-Command winget -ErrorAction SilentlyContinue)) { throw 'Rust not found. Install from https://rustup.rs/ and rerun.' }
    Write-Host 'Rust not found; installing Rustup with winget. User approval may be required.'
    Run 'winget' @('install','--id','Rustlang.Rustup','--exact','--accept-package-agreements','--accept-source-agreements')
    $env:PATH = (Join-Path $env:USERPROFILE '.cargo\bin') + ';' + $env:PATH
    $cargo = Get-Command cargo -ErrorAction SilentlyContinue
    if (-not $cargo) {throw 'Rust installation completed but cargo is not available. Open a new PowerShell and rerun.'}
}
$cargoExe = if ($cargo -is [System.Management.Automation.CommandInfo]) { $cargo.Source } else { $cargo.FullName }
$rustup = Get-Command rustup -ErrorAction SilentlyContinue
if ($rustup) {Run $rustup.Source @('target','add','x86_64-pc-windows-msvc')}
Step 'Format workspace'
Run $cargoExe @('fmt','--all')
Step 'Run unit tests (Windows x64)'
Run $cargoExe @('test','--workspace','--target','x86_64-pc-windows-msvc')
Step 'Build native release executables'
try {Run $cargoExe @('build','--workspace','--release','--target','x86_64-pc-windows-msvc')}
catch {
    Write-Warning 'If MSVC link.exe is missing, install Visual Studio 2022 Build Tools with Desktop development with C++ and Windows SDK.'
    throw
}
$version = '0.0.1'
$distribution = Join-Path $root "dist\Gestouch-$version-win-x64"
if (Test-Path $distribution) {Remove-Item -Recurse -Force $distribution}
New-Item -ItemType Directory -Force $distribution | Out-Null
$bin = Join-Path $root 'target\x86_64-pc-windows-msvc\release'
foreach ($name in @('gestouch-agent','gestouch-settings','gestouch-service')) {
    $source = Join-Path $bin "$name.exe"
    if (-not (Test-Path $source)) { throw "Build output missing: $source" }
    Copy-Item $source $distribution
}
New-Item -ItemType Directory -Force (Join-Path $distribution 'scripts') | Out-Null
New-Item -ItemType Directory -Force (Join-Path $distribution 'docs') | Out-Null
Copy-Item (Join-Path $root 'scripts\install.ps1') (Join-Path $distribution 'scripts\')
Copy-Item (Join-Path $root 'scripts\uninstall.ps1') (Join-Path $distribution 'scripts\')
Copy-Item (Join-Path $root 'README.md') $distribution
Copy-Item (Join-Path $root 'LICENSE-MIT') $distribution
Copy-Item (Join-Path $root 'LICENSE-APACHE') $distribution
Copy-Item (Join-Path $root 'docs\PRD_Gestouch_v1.0.md') (Join-Path $distribution 'docs\')
Copy-Item (Join-Path $root 'docs\WIREFRAMES.html') (Join-Path $distribution 'docs\')
$archive = Join-Path $root "dist\Gestouch-$version-win-x64-portable.zip"
if (Test-Path $archive) {Remove-Item -Force $archive}
Step 'Create portable ZIP'
Compress-Archive -Path (Join-Path $distribution '*') -DestinationPath $archive -CompressionLevel Optimal
Write-Host "`nBuilt successfully: $archive" -ForegroundColor Green
if ($Install) {
    Step 'Install'
    $installer = Join-Path $root 'scripts\install.ps1'
    & $installer -SourceDirectory $distribution -Service:$Service
    if (-not $?) {throw 'Installation failed'}
}
Write-Host 'Status: binaries produced; physical Precision Touchpad and sleep/resume tests are still required.'
