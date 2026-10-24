# Timing for phase3b-references: 5 rounds; each round runs every program once, one at a time, pinned to
# the performance cores (affinity 0xC03C03, set right after start, as target/redesign/bench/runs/run.ps1 does);
# the order is reversed on every other round so A and B alternate. Aborts unless on mains power.
$ErrorActionPreference = 'Stop'
$dir = 'C:\Users\ism19\Code\ember\target\redesign2\references\bench\out'
$out = Join-Path $dir 'results.txt'
Set-Content -Path $out -Value '' -Encoding ascii
$progs = @('words_plain','words_counted24','words_counted16','words_owned','words_rust_vecstr','words_rust_ranges','words_rust_owned',
           'get_plain','get_typed','get_cast','get_rust','get_plain_noinline','get_typed_noinline','get_cast_noinline','get_rust_noinline',
           'state','state_checked','state_rust','state_rust_inplace')
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
