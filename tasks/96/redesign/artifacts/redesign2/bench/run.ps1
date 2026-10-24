# run.ps1: the timing for tasks/96/redesign/phase3b-measurements.md.
# Usage: powershell -NoProfile -ExecutionPolicy Bypass -File run.ps1 -Out <file> -Rounds 5 -Exes a.exe,b.exe,...
# Pins ITSELF to the performance cores (mask 0xC03C03) and starts each program directly (`& $exe`), so each
# program inherits the mask from its first instruction. No cmd, no `start`, no window. One program at a time;
# forward order in odd rounds, reversed in even rounds. Mains power is checked before every round.
param([string]$Out, [int]$Rounds = 5, [string[]]$Exes)
$ErrorActionPreference = 'Stop'
$Exes = @($Exes | ForEach-Object { $_ -split "," } | Where-Object { $_ -ne "" } | ForEach-Object { (Resolve-Path $_).Path })
[System.Diagnostics.Process]::GetCurrentProcess().ProcessorAffinity = [IntPtr]0xC03C03
foreach ($r in 1..$Rounds) {
  if ((Get-CimInstance Win32_Battery).BatteryStatus -ne 2) { Add-Content $Out "ABORT not on mains before round $r"; exit 1 }
  $order = if ($r % 2 -eq 1) { $Exes } else { $Exes[($Exes.Count - 1)..0] }
  foreach ($exe in $order) {
    $line = & $exe 2>&1
    $code = $LASTEXITCODE
    $line = ($line | Out-String).Trim() -replace "`r?`n", ' | '
    Add-Content $Out "round=$r exe=$(Split-Path $exe -Leaf) exit=$code out=[$line]"
  }
}
Add-Content $Out "DONE"
