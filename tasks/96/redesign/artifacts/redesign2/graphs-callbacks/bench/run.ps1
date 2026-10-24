# run.ps1: provisional A/B timing for the graphs-and-callbacks report (phase 3b).
# Usage: powershell -NoProfile -ExecutionPolicy Bypass -File run.ps1 -Out <file> -Exes a.exe,b.exe,...
# 5 rounds. In each round every program runs once, one at a time, pinned to the performance cores
# from its start (cmd's `start /affinity C03C03`; `/b` opens no window; `/wait` runs one at a time).
# The order is forward in odd rounds and reversed in even rounds, so A and B alternate.
# Mains power is checked before every round; the run stops if the laptop is on battery.
param([string]$Out, [int]$Rounds = 5, [string[]]$Exes)
$ErrorActionPreference = 'Stop'
$Exes = @($Exes | ForEach-Object { $_ -split "," } | Where-Object { $_ -ne "" })
Set-Content -Path $Out -Value "" -Encoding ascii
foreach ($r in 1..$Rounds) {
  if ((Get-CimInstance Win32_Battery).BatteryStatus -ne 2) { Add-Content $Out "ABORT not on mains before round $r"; exit 1 }
  $order = if ($r % 2 -eq 1) { $Exes } else { $Exes[($Exes.Length - 1)..0] }
  foreach ($exe in $order) {
    $line = & cmd /c "start `"`" /b /wait /affinity C03C03 `"$exe`" 2>&1"
    $line = ($line | Out-String).Trim() -replace "`r?`n", ' | '
    Add-Content $Out "round=$r exe=$(Split-Path $exe -Leaf) out=[$line]"
  }
}
Add-Content $Out "DONE"
