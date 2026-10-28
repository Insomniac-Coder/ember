# run_wl.ps1: the lead's timing runner for the measuring phase (run 2's run.ps1, with arguments).
# Usage: powershell -NoProfile -ExecutionPolicy Bypass -File run_wl.ps1 -List lists\<wl>.tsv -Out runs\<wl>.txt -Rounds 5
# Pins ITSELF to the performance cores (mask 0xC03C03); each program inherits the mask. No cmd, no
# `start`, no window. One program at a time; forward order in odd rounds, reversed in even rounds.
# Mains power is checked before every round.
param([string]$List, [string]$Out, [int]$Rounds = 5)
$ErrorActionPreference = 'Stop'
$rows = @(Get-Content $List | Where-Object { $_.Trim() -ne "" } | ForEach-Object {
  $f = $_ -split "`t"
  [pscustomobject]@{ Label = $f[0]; Exe = $f[1]; Args = @(if ($f.Count -gt 2 -and $f[2] -ne "") { $f[2] -split " " } else { @() }) }
})
[System.Diagnostics.Process]::GetCurrentProcess().ProcessorAffinity = [IntPtr]0xC03C03
Add-Content $Out ("start=" + (Get-Date).ToUniversalTime().ToString("o"))
foreach ($r in 1..$Rounds) {
  if ((Get-CimInstance Win32_Battery).BatteryStatus -ne 2) { Add-Content $Out "ABORT not on mains before round $r"; exit 1 }
  $order = if ($r % 2 -eq 1) { $rows } else { $rows[($rows.Count - 1)..0] }
  foreach ($row in $order) {
    $exeArgs = $row.Args
    $line = & $row.Exe @exeArgs 2>&1
    $code = $LASTEXITCODE
    $line = ($line | Out-String).Trim() -replace "`r?`n", ' | '
    Add-Content $Out "round=$r label=[$($row.Label)] exit=$code out=[$line]"
  }
}
Add-Content $Out ("end=" + (Get-Date).ToUniversalTime().ToString("o"))
Add-Content $Out "DONE"
