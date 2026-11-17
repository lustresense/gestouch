# Owner-run GitHub create/push helper. DOES NOT create a release.
[CmdletBinding()]
param([string]$Repository='lustresense/gestouch',[switch]$Private)
$ErrorActionPreference='Stop'
Set-Location (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
if(-not(Get-Command git -ErrorAction SilentlyContinue)){throw 'Git is required: https://git-scm.com/downloads/win'}
if(-not(Get-Command gh -ErrorAction SilentlyContinue)){throw 'GitHub CLI required: winget install --id GitHub.cli'}
& gh auth status
if($LASTEXITCODE -ne 0){throw 'Log in first with gh auth login'}
if(-not(Test-Path '.git')){& git init -b main;if($LASTEXITCODE -ne 0){throw 'git init failed'}}
& git add .
& git diff --cached --quiet
if($LASTEXITCODE -ne 0){& git commit -m 'feat: initial Gestouch 0.0.1 Rust-native developer preview';if($LASTEXITCODE -ne 0){throw 'git commit failed; set git user.name and user.email first'}}
& gh repo view $Repository *> $null
if($LASTEXITCODE -ne 0){
  $visibility=if($Private){'--private'}else{'--public'}
  & gh repo create $Repository $visibility --source . --remote origin --push
  if($LASTEXITCODE -ne 0){throw 'GitHub repository creation failed'}
}else{
  & git remote get-url origin *> $null
  if($LASTEXITCODE -ne 0){& git remote add origin "https://github.com/$Repository.git"}
  & git push -u origin main
  if($LASTEXITCODE -ne 0){throw 'Push failed'}
}
Write-Host "Published https://github.com/$Repository — no tag or release was created."
