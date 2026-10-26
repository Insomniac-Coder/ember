param([string]$Exes, [int]$Rounds = 5, [string]$Out = "results.txt")
$list = $Exes -split ','
$Out = Join-Path $PSScriptRoot $Out; Remove-Item $Out -ErrorAction SilentlyContinue
for ($r = 1; $r -le $Rounds; $r++) {
  if ((Get-CimInstance Win32_Battery).BatteryStatus -ne 2) { "ABORT: not on mains" | Add-Content $Out; break }
  $order = if ($r % 2 -eq 0) { $c = $list.Clone(); [array]::Reverse($c); $c } else { $list }
  foreach ($e in $order) {
    $full = Join-Path $PSScriptRoot $e
    $tmp = Join-Path $PSScriptRoot "tmp_out.txt"
    $p = Start-Process -FilePath $full -NoNewWindow -PassThru -RedirectStandardOutput $tmp
    $p.ProcessorAffinity = 0xC03C03
    $p.WaitForExit()
    (Get-Content $tmp) | Add-Content $Out
  }
}
