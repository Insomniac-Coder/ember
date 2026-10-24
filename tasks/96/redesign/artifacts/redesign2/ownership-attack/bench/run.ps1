# run.ps1: provisional timing for the ownership attack report (the ownership group's run.ps1, with the
# order reversed in even rounds). Usage: run.ps1 -Out <file> -Rounds 5 -Exes a.exe,b.exe[,...]
# One program at a time, pinned to the P-cores (cmd's `start /affinity C03C03`; `/b` opens no window;
# `/wait` runs one at a time). Mains power is checked before every round; on battery the run stops.
param([string]$Out, [int]$Rounds = 5, [string[]]$Exes)
$ErrorActionPreference = 'Stop'
$Exes = @($Exes | ForEach-Object { $_ -split "," } | Where-Object { $_ -ne "" })
foreach ($r in 1..$Rounds) {
  if ((Get-CimInstance Win32_Battery).BatteryStatus -ne 2) { Add-Content $Out "ABORT not on mains before round $r"; exit 1 }
  $order = if ($r % 2 -eq 1) { $Exes } else { $Exes[($Exes.Count - 1)..0] }
  foreach ($exe in $order) {
    $line = & cmd /c "start `"`" /b /wait /affinity C03C03 `"$exe`" 2>&1"
    $line = ($line | Out-String).Trim() -replace "`r?`n", ' | '
    Add-Content $Out "round=$r exe=$(Split-Path $exe -Leaf) out=[$line]"
  }
}
Add-Content $Out "DONE"
