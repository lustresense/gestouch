# Installs for current Windows user; -Service opts in to real SCM supervisor.
[CmdletBinding()]
param([string]$SourceDirectory = '',[switch]$Service)
$ErrorActionPreference='Stop'
Set-StrictMode -Version Latest
if([string]::IsNullOrWhiteSpace($SourceDirectory)){
  $parent=(Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
  if(Test-Path (Join-Path $parent 'gestouch-agent.exe')){$SourceDirectory=$parent}
  else{$SourceDirectory=Join-Path $parent 'dist\Gestouch-0.0.1-win-x64'}
}
$identity=[Security.Principal.WindowsIdentity]::GetCurrent()
$isAdmin=(New-Object Security.Principal.WindowsPrincipal($identity)).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if ($Service -and -not $isAdmin) {throw 'Service Mode requires an elevated PowerShell (Run as Administrator).'}
$destination=if($Service){Join-Path $env:ProgramFiles 'Gestouch'}else{Join-Path $env:LOCALAPPDATA 'Programs\Gestouch'}
# Stop only the existing supervisor before replacing service executable.
if($Service){
  $prior=Get-Service -Name GestouchSupervisor -ErrorAction SilentlyContinue
  if($prior){Stop-Service -Name GestouchSupervisor -ErrorAction SilentlyContinue}
}
Get-Process gestouch-agent -ErrorAction SilentlyContinue | Where-Object { $_.Path -and $_.Path.StartsWith($destination,[System.StringComparison]::OrdinalIgnoreCase) } | Stop-Process -Force -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force $destination | Out-Null
foreach($name in @('gestouch-agent.exe','gestouch-settings.exe','gestouch-service.exe')){
  $source=Join-Path $SourceDirectory $name
  if(-not(Test-Path $source)){throw "Missing $source"}
  Copy-Item $source $destination -Force
}
$agent=Join-Path $destination 'gestouch-agent.exe'
$action=New-ScheduledTaskAction -Execute $agent
$trigger=New-ScheduledTaskTrigger -AtLogOn -User $identity.Name
$principal=New-ScheduledTaskPrincipal -UserId $identity.Name -LogonType Interactive -RunLevel Limited
$settings=New-ScheduledTaskSettingsSet -MultipleInstances IgnoreNew -ExecutionTimeLimit (New-TimeSpan -Seconds 0) -StartWhenAvailable -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries
Register-ScheduledTask -TaskName 'GestouchAgent' -Action $action -Trigger $trigger -Principal $principal -Settings $settings -Force | Out-Null
Start-ScheduledTask -TaskName 'GestouchAgent'
if($Service){
  $serviceExe=Join-Path $destination 'gestouch-service.exe'
  $exists=Get-Service -Name GestouchSupervisor -ErrorAction SilentlyContinue
  if($exists){
    Stop-Service -Name GestouchSupervisor -ErrorAction SilentlyContinue
    & sc.exe config GestouchSupervisor binPath= "`"$serviceExe`"" start= auto | Out-Null
  }else{
    & sc.exe create GestouchSupervisor binPath= "`"$serviceExe`"" start= auto obj= LocalSystem DisplayName= 'Gestouch Supervisor' | Out-Null
  }
  if($LASTEXITCODE -ne 0){throw "sc.exe failed: $LASTEXITCODE"}
  Start-Service -Name GestouchSupervisor
}
Write-Host "Gestouch installed in $destination. Agent task enabled for $($identity.Name)."
if($Service){Write-Host 'Service Mode experimental: supervisor invokes the logged-on user task; gesture engine stays in the user session.'}
