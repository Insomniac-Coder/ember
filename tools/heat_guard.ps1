# The owner, 2026-11-19: "if the CPU temp hits 102 more than 3 times you pause all things and wait for it
# to get down to a reasonable temperature and then you resume".
#
# Reads the CPU temperature HWiNFO publishes (HKCU\Software\HWiNFO64\VSB: CPU Package, Core Max). One climb
# to -Hot counts once; it counts again only after falling below -Rearm. The hit after -Allowed suspends every
# build and test process of this repository (nothing is lost: each carries on where it stopped), keeps
# suspending any new one, and resumes them all once the CPU has stayed at -CoolTo for -Hold seconds. A laptop
# with a browser open idles in the 70s and low 80s, so after -FallbackAfter seconds -CoolFallback is enough.
# ponytail: GCC running inside WSL is not suspended; add it if the sweeps turn out to matter.
#
#   powershell -File tools\heat_guard.ps1              run the guard
#   powershell -File tools\heat_guard.ps1 -ResumeAll   resume what a guard that died left suspended
param(
    [int]$Hot = 102, [int]$Rearm = 100, [int]$Allowed = 3,
    [int]$CoolTo = 80, [int]$CoolFallback = 85, [int]$FallbackAfter = 600, [int]$Hold = 30,
    [int]$Only = 0, [switch]$Once, [switch]$ResumeAll
)
$ErrorActionPreference = 'Continue'
$root = Split-Path -Parent $PSScriptRoot
$log = Join-Path $root 'target\heat-guard.log'
$record = Join-Path $root 'target\heat-guard-suspended.txt'

Add-Type -Namespace HeatGuard -Name Native -MemberDefinition @'
[DllImport("kernel32.dll", SetLastError = true)] public static extern System.IntPtr OpenProcess(uint access, bool inherit, int pid);
[DllImport("kernel32.dll")] public static extern bool CloseHandle(System.IntPtr handle);
[DllImport("ntdll.dll")] public static extern int NtSuspendProcess(System.IntPtr handle);
[DllImport("ntdll.dll")] public static extern int NtResumeProcess(System.IntPtr handle);
'@

function Say($text) {
    $line = "$(Get-Date -Format 'yyyy-MM-dd HH:mm:ss') $text"
    Add-Content -Path $log -Value $line -Encoding UTF8
    Write-Output $line
}

function Temperature {
    try {
        $p = Get-ItemProperty 'HKCU:\Software\HWiNFO64\VSB' -ErrorAction Stop
        return [Math]::Max([double]$p.ValueRaw0, [double]$p.ValueRaw1)
    } catch { return $null }
}

function Act($processId, [bool]$suspend) {
    $handle = [HeatGuard.Native]::OpenProcess(0x0800, $false, $processId)   # PROCESS_SUSPEND_RESUME
    if ($handle -eq [IntPtr]::Zero) { return $false }
    $status = if ($suspend) { [HeatGuard.Native]::NtSuspendProcess($handle) } else { [HeatGuard.Native]::NtResumeProcess($handle) }
    [void][HeatGuard.Native]::CloseHandle($handle)
    return $status -eq 0
}

function Targets {
    if ($Only) { return @($Only) }
    $tools = 'cargo', 'rustc', 'rustup', 'ember', 'clang', 'clang++', 'clang-cl', 'lld-link', 'rust-lld', 'link', 'cl', 'mspdbsrv', 'llc', 'opt'
    $repo = "$root\"
    $temp = Join-Path $env:TEMP 'ember-'
    Get-CimInstance Win32_Process | Where-Object {
        $name = $_.Name -replace '\.exe$', ''
        $path = [string]$_.ExecutablePath
        $_.ProcessId -ne $PID -and (
            $tools -contains $name -or
            $path.StartsWith($repo, 'OrdinalIgnoreCase') -or $path.StartsWith($temp, 'OrdinalIgnoreCase') -or
            ($name -eq 'python' -and [string]$_.CommandLine -match 'annotations\.py|qc_again|a_migrate|tools[\\/]|tasks[\\/]')
        )
    } | ForEach-Object { $_.ProcessId }
}

function Resume-All {
    if (-not (Test-Path $record)) { return 0 }
    $count = 0
    foreach ($id in Get-Content $record | Sort-Object -Unique) { if ($id -and (Act ([int]$id) $false)) { $count++ } }
    Remove-Item $record -Force
    return $count
}

if ($ResumeAll) { Say "resumed $(Resume-All) process(es) by hand"; exit 0 }

function Pause-Until-Cool($hits) {
    $start = Get-Date
    $cool = $null
    $held = @{}
    Say "PAUSE: $hits hits of $Hot C; suspending the builds and tests until the CPU cools"
    while ($true) {
        foreach ($id in Targets) {
            if (-not $held.ContainsKey($id) -and (Act $id $true)) { $held[$id] = $true; Add-Content -Path $record -Value $id }
        }
        $t = Temperature
        $limit = if (((Get-Date) - $start).TotalSeconds -gt $FallbackAfter) { $CoolFallback } else { $CoolTo }
        if ($null -ne $t -and $t -le $limit) {
            if (-not $cool) { $cool = Get-Date }
            if (((Get-Date) - $cool).TotalSeconds -ge $Hold) { break }
        } else { $cool = $null }
        Start-Sleep -Seconds 2
    }
    $n = Resume-All
    Say "RESUME: $n process(es) resumed at $t C after $([int]((Get-Date) - $start).TotalSeconds) s"
}

[void](Resume-All)
$hits = 0
$armed = $true
$blind = $false
Say "guard on: pause after more than $Allowed hits of $Hot C, resume at $CoolTo C held $Hold s ($CoolFallback C after $FallbackAfter s)"
try {
    while ($true) {
        $t = Temperature
        if ($null -eq $t) {
            if (-not $blind) { Say 'NO TEMPERATURE: HWiNFO is not publishing it, so nothing is guarded'; $blind = $true }
        } else {
            $blind = $false
            if ($armed -and $t -ge $Hot) {
                $hits++
                $armed = $false
                Say "hit $hits at $t C"
            } elseif (-not $armed -and $t -lt $Rearm) { $armed = $true }
            if ($hits -gt $Allowed) {
                Pause-Until-Cool $hits
                $hits = 0
                $armed = $true
                if ($Once) { break }
            }
        }
        Start-Sleep -Seconds 1
    }
} finally {
    [void](Resume-All)
}
