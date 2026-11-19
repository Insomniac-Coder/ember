# The push gate (HANDOFF "Push gate"), stop tolerant: the suite three ways (as the CI job runs it: Clang, the LLVM
# backend, Microsoft's linker forced, an empty cache; the LLVM default with MSVC on the efficiency cores, with the
# release build, the gates and the C warning sweeps; the C backend), each split into pieces (one test target, one
# gate, one sweep). A piece counts as done only once it has finished, and its result is kept under
# target\push-gate\<key>\, where the key is the commit and the uncommitted changes: run the gate again after a stop
# and it goes on from the first piece not done; change the tree and it starts over. -Fresh starts over anyway.
#
#   powershell -NoProfile -ExecutionPolicy Bypass -File tools\push_gate.ps1 [-Fresh] [-Stages ci_style,full,c_backend]
#
# The summary is target\push-gate\summary.txt (UTF-8); the exit code is 0 when every piece passed.
param(
    [switch]$Fresh,
    [string[]]$Stages = @('ci_style', 'full', 'c_backend'),
    # Only these test targets (`--lib`, `--test milestones`, ...): for trying the script on a few.
    [string[]]$Units = @()
)
$ErrorActionPreference = 'Continue'
# `-File` gives a list as one string: `-Stages ci_style,full` and `-Units "--lib,--test ui"` are split here.
$Stages = @($Stages | ForEach-Object { $_ -split ',' } | Where-Object { $_ })
$Units = @($Units | ForEach-Object { $_ -split ',' } | Where-Object { $_ })
$root = Split-Path -Parent $PSScriptRoot
Set-Location $root
$env:CARGO_BUILD_JOBS = '4'
$env:PYTHONIOENCODING = 'utf-8'

# The key: the commit and every change not committed (tracked files only; a new test file is committed first).
$head = (git rev-parse HEAD).Trim()
$changes = (git diff HEAD | Out-String) + (git status --porcelain=v1 | Out-String)
# A new file not yet committed is part of the tree too: its contents count.
foreach ($file in @(git ls-files --others --exclude-standard)) {
    if (Test-Path -LiteralPath $file -PathType Leaf) { $changes += "$file $((Get-FileHash -LiteralPath $file).Hash)`n" }
}
$sha = [System.Security.Cryptography.SHA256]::Create()
$digest = $sha.ComputeHash([System.Text.Encoding]::UTF8.GetBytes("$head`n$changes"))
$key = ($head.Substring(0, 8) + '-' + (-join ($digest[0..3] | ForEach-Object { $_.ToString('x2') })))
$base = Join-Path $root 'target\push-gate'
$state = Join-Path $base $key
if ($Fresh -and (Test-Path $state)) { Remove-Item -Recurse -Force $state }
New-Item -ItemType Directory -Force "$state\done", "$state\logs" | Out-Null
# Older keys are other trees: their results say nothing about this one.
Get-ChildItem $base -Directory | Where-Object { $_.Name -ne $key } | Remove-Item -Recurse -Force -ErrorAction SilentlyContinue

function Done([string]$name) { Join-Path "$state\done" (($name -replace '[^A-Za-z0-9_.-]+', '_') + '.txt') }
function Log([string]$name) { Join-Path "$state\logs" (($name -replace '[^A-Za-z0-9_.-]+', '_') + '.txt') }

# Runs `exe args`, its output (UTF-8, as the program writes it) in `log`; gives the exit code.
function Run-Logged([string]$exe, [string[]]$arguments, [string]$log) {
    $p = Start-Process -FilePath $exe -ArgumentList $arguments -RedirectStandardOutput $log -RedirectStandardError "$log.err" `
        -NoNewWindow -PassThru
    $null = $p.Handle
    $p.WaitForExit()
    if (Test-Path "$log.err") {
        Get-Content "$log.err" -Encoding UTF8 | Add-Content -Path $log -Encoding UTF8
        Remove-Item "$log.err"
    }
    return $p.ExitCode
}

# One piece: skipped when its result is kept, else run, and its one-line result kept once it has finished.
$script:planned = New-Object System.Collections.Generic.List[string]
function Step([string]$name, [scriptblock]$run) {
    $script:planned.Add($name)
    $done = Done $name
    if (Test-Path $done) {
        Write-Host "kept  $name : $(Get-Content $done -Encoding UTF8 -TotalCount 1)"
        return
    }
    Write-Host "run   $name ($(Get-Date -Format T))"
    $line = (& $run | Out-String).Trim()
    # Written whole, then renamed: a stop while writing leaves no half result.
    $line | Out-File -Encoding utf8 "$done.tmp"
    Move-Item -Force "$done.tmp" $done
    Write-Host "done  $name : $line"
}

# The test targets of the workspace, as `cargo test --workspace` runs them: every library's and binary's own
# tests, each integration test target, the doc tests.
if ($Units.Count -eq 0) {
    $meta = cargo metadata --no-deps --format-version 1 --locked | ConvertFrom-Json
    $tests = @($meta.packages | ForEach-Object { $_.targets } | Where-Object { $_.kind -contains 'test' } |
        ForEach-Object { $_.name } | Sort-Object -Unique)
    $Units = @('--lib', '--bins') + @($tests | ForEach-Object { "--test $_" }) + @('--doc')
}

function Suite([string]$stage) {
    foreach ($unit in $Units) {
        Step "$stage $unit" {
            $log = Log "$stage $unit"
            $arguments = @('test', '--workspace', '--locked', '--profile', 'ci', '--no-fail-fast') + $unit.Split(' ')
            $code = Run-Logged 'cargo' $arguments $log
            $failed = @(Select-String -Path $log -Pattern '^test .* FAILED\s*$' | ForEach-Object { $_.Line.Trim() }) -join '; '
            "exit $code $failed"
        }
    }
}

# 1. As the CI job runs it: an empty cache when the stage starts, kept across a stop.
if ($Stages -contains 'ci_style') {
    $started = @(Get-ChildItem "$state\done" -Filter 'ci_style_*' -ErrorAction SilentlyContinue).Count -gt 0
    if (-not $started) { Remove-Item -Recurse -Force target\ci-cache -ErrorAction SilentlyContinue }
    $env:EMBER_CC = 'clang'; $env:EMBER_BACKEND = 'llvm'; $env:EMBER_CACHE = 'target/ci-cache'
    $env:CCC_OVERRIDE_OPTIONS = '# ^-fuse-ld=link ^-Wno-unused-command-line-argument'
    $env:RUST_TEST_THREADS = '12'
    Suite 'ci_style'
    Remove-Item Env:\EMBER_CC, Env:\EMBER_BACKEND, Env:\EMBER_CACHE, Env:\CCC_OVERRIDE_OPTIONS -ErrorAction SilentlyContinue
}

# 2. The LLVM default with MSVC on the efficiency cores; the release build the gates and sweeps run; the gates;
#    the C warning sweeps (gcc's in WSL).
if ($Stages -contains 'full') {
    $self = [System.Diagnostics.Process]::GetCurrentProcess()
    $affinity = $self.ProcessorAffinity
    $self.ProcessorAffinity = [IntPtr]0x3FC3FC
    $env:RUST_TEST_THREADS = '16'
    Suite 'full'
    $self.ProcessorAffinity = $affinity
    Step 'release build' {
        $code = Run-Logged 'cargo' @('build', '-p', 'ember_driver', '--bin', 'ember', '--release') (Log 'release build')
        "exit $code"
    }
    $gates = @(
        'tools\hardening_check.py',
        'tools\split_spec.py --check docs\spec-source\ember-spec.md docs\spec',
        'tools\rule_index.py', 'tools\check_branding.py', 'tools\generate_runtime.py --check',
        'tools\test_runtime_generation.py', 'tools\test_annotation_cleanup.py', 'tools\gen_math_vectors.py --check',
        'tools\spec_check.py', 'tools\error_pages.py', 'tools\check_det.py', 'tools\test_rule_index.py',
        'tools\unicode_case.py --check', 'tools\phase_progress.py --check'
    )
    foreach ($gate in $gates) {
        Step "gate $gate" {
            $log = Log "gate $gate"
            $code = Run-Logged 'python' $gate.Split(' ') $log
            $last = (Get-Content $log -Encoding UTF8 -Tail 1)
            "exit $code $last"
        }
    }
    Step 'emit-appendix' {
        $before = (Get-FileHash tasks\spec-0.9.9\parts\p19-annex-a.md).Hash
        $null = Run-Logged 'python' @('tools\spec_check.py', '--emit-appendix') (Log 'emit-appendix')
        $after = (Get-FileHash tasks\spec-0.9.9\parts\p19-annex-a.md).Hash
        "unchanged: $($before -eq $after)"
    }
    foreach ($cc in 'msvc', 'clang') {
        Step "sweep $cc" {
            $log = Log "sweep $cc"
            $code = Run-Logged 'python' @('tools\c_warning_sweep.py', $cc, '--jobs', '12') $log
            "exit $code $((Get-Content $log -Encoding UTF8 -Tail 1))"
        }
    }
    Step 'sweep gcc' {
        $log = Log 'sweep gcc'
        $script = (wsl wslpath -a ($root -replace '\\', '/')).Trim() + '/tools/gcc_sweep.sh'
        # One argument with its own quotes: Start-Process joins its arguments with spaces.
        $code = Run-Logged 'wsl' @('bash', '-c', "`"tr -d '\r' < '$script' > /tmp/gs.sh && GATE_REPO='$script' bash /tmp/gs.sh`"") $log
        "exit $code $((Get-Content $log -Encoding UTF8 -Tail 1))"
    }
}

# 3. The C backend.
if ($Stages -contains 'c_backend') {
    $env:EMBER_BACKEND = 'c'
    $env:RUST_TEST_THREADS = '12'
    Suite 'c_backend'
    Remove-Item Env:\EMBER_BACKEND -ErrorAction SilentlyContinue
}

# The summary: every kept result, in order; a piece passed when it exited 0 (a sweep, also with no warning).
$lines = @("push gate on $head (+ the tree), key $key, $(Get-Date -Format 'yyyy-MM-dd HH:mm')")
$bad = 0
foreach ($name in $script:planned) {
    $done = Done $name
    $text = if (Test-Path $done) { (Get-Content $done -Encoding UTF8 -TotalCount 1) } else { 'not run' }
    $ok = $text -match '^exit 0\b' -or $text -match '^unchanged: True'
    if ($name -like 'sweep *') { $ok = $ok -and $text -match ': 0 of ' }
    if (-not $ok) { $bad += 1 }
    $lines += ('{0} {1}: {2}' -f ($(if ($ok) { 'ok  ' } else { 'FAIL' }), $name, $text))
}
$lines += "$bad failing piece(s)"
$lines | Out-File -Encoding utf8 (Join-Path $base 'summary.txt')
$lines | ForEach-Object { Write-Host $_ }
exit $(if ($bad -eq 0) { 0 } else { 1 })
