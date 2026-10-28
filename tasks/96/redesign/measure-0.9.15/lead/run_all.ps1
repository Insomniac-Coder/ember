# run_all.ps1: every workload of the measuring phase, one after another (5 rounds each).
param([string]$Suffix = "")
$ErrorActionPreference = 'Stop'
Set-Location $PSScriptRoot
New-Item -ItemType Directory -Force runs | Out-Null
foreach ($l in Get-ChildItem lists\*.tsv) {
  $wl = $l.BaseName
  $out = "runs\$wl$Suffix.txt"
  if (Test-Path $out) { Remove-Item $out }
  & powershell -NoProfile -ExecutionPolicy Bypass -File run_wl.ps1 -List $l.FullName -Out $out -Rounds 5
  if ($LASTEXITCODE -ne 0) { "STOPPED at $wl" | Add-Content runs\all$Suffix.txt; exit 1 }
  "$wl done" | Add-Content runs\all$Suffix.txt
}
"ALLDONE" | Add-Content runs\all$Suffix.txt
