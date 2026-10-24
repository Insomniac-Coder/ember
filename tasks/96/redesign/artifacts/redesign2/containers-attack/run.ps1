# Timing runner for phase 3b (containers): 5 rounds; in each round every program runs once, one at a
# time, pinned to the performance cores (0xC03C03). The order alternates: forward in odd rounds,
# reversed in even rounds. Aborts if the laptop is not on mains power before a round.
# Usage: powershell -File run.ps1 <results file> <exe> [<exe> ...]
param([string]$out, [Parameter(ValueFromRemainingArguments=$true)][string[]]$exes)
$ErrorActionPreference = 'Stop'
Set-Content -Path $out -Value '' -Encoding ascii
$tmpo = "$out.tmp_out"; $tmpe = "$out.tmp_err"
foreach ($r in 1..5) {
  if ((Get-CimInstance Win32_Battery).BatteryStatus -ne 2) { Add-Content $out "ABORT not on mains before round $r"; exit 1 }
  $order = if ($r % 2 -eq 1) { $exes } else { $exes[($exes.Length - 1)..0] }
  foreach ($exe in $order) {
    $p = Start-Process $exe -NoNewWindow -PassThru -RedirectStandardOutput $tmpo -RedirectStandardError $tmpe
    try { $p.ProcessorAffinity = 0xC03C03 } catch { Add-Content $out "WARN affinity not set round=$r $exe" }
    $p.WaitForExit()
    $line = (Get-Content $tmpo -Raw); if ($line) { $line = $line.Trim() }
    $err = (Get-Content $tmpe -Raw); if ($err) { $err = $err.Trim() }
    Add-Content $out "round=$r exit=$($p.ExitCode) out=[$line] err=[$err]"
  }
}
Remove-Item $tmpo, $tmpe -ErrorAction SilentlyContinue
Add-Content $out "DONE"
