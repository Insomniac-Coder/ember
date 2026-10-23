# Phase 3 timing: 5 rounds; in each round every program runs once, one at a time, pinned to 0xC03C03.
$ErrorActionPreference = 'Stop'
$bench = 'C:\Users\ism19\Code\ember\target\redesign\bench'
$out = Join-Path $bench 'runs\results.txt'
Set-Content -Path $out -Value '' -Encoding ascii
$progs = @()
foreach ($w in 'w1','w2','w3','w4') {
  foreach ($m in 'plain','D1','D2','D3') { $progs += ,@($m, $w, "$m\$w.exe") }
  if ($w -eq 'w3') { foreach ($v in 'w3_sweep','w3_malloc','w3_pool') { $progs += ,@("plain-$v", 'w3', "plain\$v.exe") } }
}
foreach ($r in 1..5) {
  if ((Get-CimInstance Win32_Battery).BatteryStatus -ne 2) { Add-Content $out "ABORT not on mains before round $r"; exit 1 }
  foreach ($pg in $progs) {
    $m = $pg[0]; $w = $pg[1]; $exe = Join-Path $bench $pg[2]
    $o = Join-Path $bench 'runs\tmp_out.txt'; $e = Join-Path $bench 'runs\tmp_err.txt'
    $p = Start-Process $exe -NoNewWindow -PassThru -RedirectStandardOutput $o -RedirectStandardError $e
    try { $p.ProcessorAffinity = 0xC03C03 } catch { Add-Content $out "WARN affinity not set round=$r $m $w" }
    $p.WaitForExit()
    $line = (Get-Content $o -Raw); if ($line) { $line = $line.Trim() }
    $err = (Get-Content $e -Raw); if ($err) { $err = $err.Trim() }
    Add-Content $out "round=$r model=$m workload=$w out=[$line] err=[$err]"
  }
}
Add-Content $out "DONE"
