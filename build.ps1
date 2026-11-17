# One-command Gestouch 0.0.1 build entrypoint for Windows PowerShell.
[CmdletBinding()]
param([switch]$Install,[switch]$Service)
$ErrorActionPreference='Stop'
& (Join-Path $PSScriptRoot 'scripts\build.ps1') -Install:$Install -Service:$Service
if(-not $?) {throw 'Gestouch build failed'}
