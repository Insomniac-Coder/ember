# run.ps1: provisional A/B timing for the ownership report.
# Usage: run.ps1 -Out <file> -Rounds 5 -Exes a.exe,b.exe[,...]
# Each round runs every program once, in the given order, one at a time, pinned to the P-cores
# (cmd's `start /affinity C03C03`, `/b` so no window opens, `/wait` so only one runs at a time).
# Mains power is checked before every round; the run stops if the laptop is on battery.
param([string]$Out, [int]$Rounds = 5, [string[]]$Exes)
$ErrorActionPreference = 'Stop'
$Exes = @($Exes | ForEach-Object { $_ -split "," } | Where-Object { $_ -ne "" })
foreach ($r in 1..$Rounds) {
  if ((Get-CimInstance Win32_Battery).BatteryStatus -ne 2) { Add-Content $Out "ABORT not on mains before round $r"; exit 1 }
  foreach ($exe in $Exes) {
    $line = & cmd /c "start `"`" /b /wait /affinity C03C03 `"$exe`" 2>&1"
    $line = ($line | Out-String).Trim() -replace "`r?`n", ' | '
    Add-Content $Out "round=$r exe=$(Split-Path $exe -Leaf) out=[$line]"
  }
}
Add-Content $Out "DONE"
