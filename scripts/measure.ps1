# Sample Gestouch agent RAM and CPU usage (requires agent running).
[CmdletBinding()]
param([int]$Seconds=120,[int]$IntervalSeconds=2,[string]$Output='gestouch-benchmark.csv')
$ErrorActionPreference='Stop'
$records=New-Object System.Collections.Generic.List[object]
$prev=$null
for($i=0;$i -lt $Seconds;$i+=$IntervalSeconds){
  $p=Get-Process -Name gestouch-agent -ErrorAction SilentlyContinue | Select-Object -First 1
  if(-not $p){throw 'gestouch-agent is not running'}
  $now=Get-Date
  $cpu=$null
  if($null -ne $prev){$cpu=[math]::Round((($p.CPU-$prev.Cpu)/($now-$prev.Time).TotalSeconds)*100/[Environment]::ProcessorCount,3)}
  $records.Add([pscustomobject]@{Timestamp=$now.ToString('o');Pid=$p.Id;WorkingSetMiB=[math]::Round($p.WorkingSet64/1MB,3);PrivateMiB=[math]::Round($p.PrivateMemorySize64/1MB,3);CpuPctMachine=$cpu})
  $prev=@{Cpu=$p.CPU;Time=$now}
  Start-Sleep -Seconds $IntervalSeconds
}
$records | Export-Csv -NoTypeInformation -Encoding UTF8 -Path $Output
Write-Host "Saved $($records.Count) rows to $Output. Inspect idle vs active phases separately."
