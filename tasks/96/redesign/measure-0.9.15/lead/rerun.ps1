# rerun.ps1: two more 5-round sets (suffixes b and c) of the workloads whose first set had a program over 10% spread.
param([string[]]$Workloads)
$ErrorActionPreference = 'Stop'
Set-Location $PSScriptRoot
$Workloads = @($Workloads | ForEach-Object { $_ -split "," } | Where-Object { $_ -ne "" })
foreach ($suffix in "b", "c") {
  foreach ($wl in $Workloads) {
    $out = "runs\$wl$suffix.txt"
    if (Test-Path $out) { Remove-Item $out }
    & powershell -NoProfile -ExecutionPolicy Bypass -File run_wl.ps1 -List "lists\$wl.tsv" -Out $out -Rounds 5
    if ($LASTEXITCODE -ne 0) { "STOPPED at $wl$suffix" | Add-Content runs\rerun.txt; exit 1 }
    "$wl$suffix done" | Add-Content runs\rerun.txt
  }
}
"ALLDONE" | Add-Content runs\rerun.txt
