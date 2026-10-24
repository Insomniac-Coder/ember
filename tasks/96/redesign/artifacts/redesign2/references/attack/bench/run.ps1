# Timing for phase3b-references-attack.md, the same method as ../../bench/run.ps1: 5 rounds; each round runs every
# program once, one at a time, pinned to the performance cores (affinity 0xC03C03, set right after start); the order
# is reversed on every other round so A and B alternate. Aborts unless on mains power.
# Run: powershell -NoProfile -ExecutionPolicy Bypass -File run.ps1 ; then python -I ../../bench/summarize.py out/results.txt
$ErrorActionPreference = 'Stop'
$dir = 'C:\Users\ism19\Code\ember\target\redesign2\references\attack\bench\out'
$out = Join-Path $dir 'results.txt'
Set-Content -Path $out -Value '' -Encoding ascii
$progs = @('words_plain','words_ember','words_rust_vecstr',
           'state','state_checked','state_inplace','state_rust','state_rust_inplace',
           'keep_reuse_short','keep_copy_short','keep_shrink_short','keep_half_short',
           'keep_reuse_trim','keep_copy_trim','keep_half_trim',
           'keep_reuse_long','keep_copy_long','keep_half_long',
           'get_plain','get_typed','get_typed_slotvar','get_cast','get_rust')
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
