# Runs each Python twin once (not a timing), pinned to the efficiency cores (0x3FC3FC) so it stays off
# the performance cores other agents time on. Output: <name>_output.txt beside each file.
$dir = Split-Path -Parent $MyInvocation.MyCommand.Path
foreach ($name in @('w2', 'w3', 'w4', 'w1_dicts', 'w1')) {
    $out = Join-Path $dir "${name}_output.txt"
    $p = Start-Process -FilePath 'C:\Python314\python.exe' -ArgumentList @('-I', (Join-Path $dir "$name.py")) `
        -NoNewWindow -PassThru -RedirectStandardOutput $out -RedirectStandardError (Join-Path $dir "${name}_stderr.txt")
    $p.ProcessorAffinity = [IntPtr]0x3FC3FC
    $p.WaitForExit()
    "$name exit=$($p.ExitCode) output=$((Get-Content $out) -join ' ')"
}
