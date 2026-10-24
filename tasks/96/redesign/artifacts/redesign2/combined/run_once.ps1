# Runs each program ONCE (this is not a timing: the ms= field each prints is not used), pinned to the
# efficiency cores (0x3FC3FC) so it stays off the performance cores other agents time on, one at a
# time, started directly by PowerShell (no cmd, no start, no window). Prints the program's own line.
$here = Split-Path -Parent $MyInvocation.MyCommand.Path
$r2 = Split-Path -Parent $here
$progs = @(
  "$here\w1.exe", "$here\w2.exe", "$here\w3.exe", "$here\w3_registration.exe", "$here\w4.exe",
  "$here\w1_cursor_check.exe", "$here\w1_cursor_check_norule.exe", "$here\w2_link_check.exe",
  "$here\w3_poison.exe", "$here\w4_poison.exe",
  "$r2\salvage\bin\plain_w1_group.exe", "$r2\graphs-callbacks-attack2\bench\out\w2_plain_slab_array24.exe",
  "$r2\salvage\bin\plain_w3.exe", "$r2\graphs-callbacks-attack2\bench\out\w4_plain_same_slab.exe",
  "$here\first-run\w2.exe", "$here\first-run\w3.exe", "$here\first-run\w3_registration.exe"
)
if ((Get-CimInstance Win32_Battery).BatteryStatus -ne 2) { "not on mains power: stopped"; exit 1 }
foreach ($exe in $progs) {
    $out = [System.IO.Path]::GetTempFileName(); $err = [System.IO.Path]::GetTempFileName()
    $p = Start-Process -FilePath $exe -NoNewWindow -PassThru -RedirectStandardOutput $out -RedirectStandardError $err
    $null = $p.Handle
    try { $p.ProcessorAffinity = [IntPtr]0x3FC3FC } catch { "affinity not set for $exe" }
    $p.WaitForExit()
    $o = ((Get-Content $out) + (Get-Content $err)) -join ' | '
    "$(Split-Path -Leaf (Split-Path -Parent $exe))/$(Split-Path -Leaf $exe) exit=$($p.ExitCode): $o"
    Remove-Item $out, $err
}
