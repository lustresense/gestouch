# Removes auto-start + optional supervisor. Keeps user rules/config by default.
[CmdletBinding()]
param([switch]$RemoveConfig)
$ErrorActionPreference='Stop'
$service=Get-Service GestouchSupervisor -ErrorAction SilentlyContinue
if($service){
  $isAdmin=(New-Object Security.Principal.WindowsPrincipal([Security.Principal.WindowsIdentity]::GetCurrent())).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
  if(-not $isAdmin){throw 'Run elevated PowerShell to remove Windows Service.'}
  Stop-Service GestouchSupervisor -ErrorAction SilentlyContinue
  & sc.exe delete GestouchSupervisor
  if($LASTEXITCODE -ne 0){throw 'Could not delete service.'}
}
$task=Get-ScheduledTask -TaskName GestouchAgent -ErrorAction SilentlyContinue
if($task){Stop-ScheduledTask -TaskName GestouchAgent -ErrorAction SilentlyContinue;Unregister-ScheduledTask -TaskName GestouchAgent -Confirm:$false}
Get-Process gestouch-agent,gestouch-settings -ErrorAction SilentlyContinue | Stop-Process -ErrorAction SilentlyContinue
foreach($dir in @((Join-Path $env:LOCALAPPDATA 'Programs\Gestouch'),(Join-Path $env:ProgramFiles 'Gestouch'))){
  if(Test-Path $dir){try{Remove-Item -Recurse -Force $dir}catch{Write-Warning "Could not delete $dir; close all Gestouch processes first."}}
}
if($RemoveConfig){Remove-Item -Recurse -Force (Join-Path $env:LOCALAPPDATA 'Gestouch') -ErrorAction SilentlyContinue}
Write-Host 'Gestouch uninstall completed. Config retained unless -RemoveConfig was specified.'
