# Timing for the second attack on phase3b-references.md: 5 rounds; each round runs every program once, one at a
# time, pinned to the performance cores (affinity 0xC03C03, set right after start, as the shared runner does);
# the order is reversed on every other round so A and B alternate. Aborts unless on mains power.
# Run: powershell -NoProfile -ExecutionPolicy Bypass -File run.ps1 ; then python -I ../../bench/summarize.py out/results.txt
$ErrorActionPreference = 'Stop'
$dir = 'C:\Users\ism19\Code\ember\target\redesign2\references\attack2\bench\out'
$out = Join-Path $dir 'results.txt'
Set-Content -Path $out -Value '' -Encoding ascii
$progs = @('get_plain','get_typed','get_cast','get2_table','get2_table_slotvar','get2_table_slothoist','get2_cast','get_rust',
           'keep2_half_len','keep2_half_cap','keep2_copy')
foreach ($r in 1..5) {
  if ((Get-CimInstance Win32_Battery).BatteryStatus -ne 2) { Add-Content $out "ABORT not on mains before round $r"; exit 1 }
  $order = if ($r % 2 -eq 0) { $progs[($progs.Count-1)..0] } else { $progs }
  foreach ($pg in $order) {
    $exe = Join-Path $dir "$pg.exe"
    $o = Join-Path $dir 'tmp_out.txt'; $e = Join-Path $dir 'tmp_err.txt'
    $p = Start-Process $exe -NoNewWindow -PassThru -RedirectStandardOutput $o -RedirectStandardError $e
    try { $p.ProcessorAffinity = 0xC03C03 } catch { Add-Content $out "WARN affinity not set round=$r $pg" }
    $p.WaitForExit()
    $line = (Get-Content $o -Raw); if ($line) { $line = $line.Trim() }
    $err = (Get-Content $e -Raw); if ($err) { $err = $err.Trim() }
    Add-Content $out "round=$r prog=$pg out=[$line] err=[$err]"
  }
}
Add-Content $out "DONE"
