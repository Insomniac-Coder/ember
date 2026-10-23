# `[TOOL-1]` — Ember's one-line installer for Windows x64:
#
#     irm https://github.com/Insomniac-Coder/ember/releases/latest/download/install.ps1 | iex
#
# It downloads the Windows toolchain archive from the release, checks it against the release's
# checksum, unpacks it into `$env:EMBER_HOME` (default `%LOCALAPPDATA%\Ember`, replacing an earlier
# install) and puts its `bin` on the user's `PATH`. `$env:EMBER_VERSION = 'v0.1.0'` installs that
# release instead of the latest. The archive carries Clang and LLD; the Windows SDK's libraries,
# which their licence keeps out of it, come with the Visual Studio Build Tools (`[TOOL-2]`).
$ErrorActionPreference = 'Stop'

$repository = 'Insomniac-Coder/ember'
$home_dir = if ($env:EMBER_HOME) { $env:EMBER_HOME } else { Join-Path $env:LOCALAPPDATA 'Ember' }
if (-not [Environment]::Is64BitOperatingSystem) {
    throw 'ember: the Windows archive is for x64'
}
$base = if ($env:EMBER_VERSION) {
    "https://github.com/$repository/releases/download/$($env:EMBER_VERSION)"
} else {
    "https://github.com/$repository/releases/latest/download"
}
$archive = 'ember-windows-x64.zip'

$work = Join-Path ([IO.Path]::GetTempPath()) ([Guid]::NewGuid().ToString())
New-Item -ItemType Directory -Path $work | Out-Null
try {
    Write-Host "downloading $archive"
    $ProgressPreference = 'SilentlyContinue'
    Invoke-WebRequest -UseBasicParsing -Uri "$base/$archive" -OutFile (Join-Path $work $archive)
    Invoke-WebRequest -UseBasicParsing -Uri "$base/$archive.sha256" -OutFile (Join-Path $work "$archive.sha256")
    $expected = ((Get-Content (Join-Path $work "$archive.sha256") -Raw).Trim() -split '\s+')[0].ToLowerInvariant()
    $actual = (Get-FileHash -Algorithm SHA256 (Join-Path $work $archive)).Hash.ToLowerInvariant()
    if ($expected -ne $actual) {
        throw "ember: $archive does not match the release's checksum; nothing was installed"
    }
    Expand-Archive -Path (Join-Path $work $archive) -DestinationPath $work
    if (Test-Path $home_dir) {
        Remove-Item -Recurse -Force $home_dir
    }
    New-Item -ItemType Directory -Force -Path (Split-Path $home_dir) | Out-Null
    Move-Item (Join-Path $work 'ember') $home_dir
} finally {
    Remove-Item -Recurse -Force $work -ErrorAction SilentlyContinue
}

$bin = Join-Path $home_dir 'bin'
$user_path = [Environment]::GetEnvironmentVariable('Path', 'User')
$entries = if ($user_path) { $user_path -split ';' } else { @() }
if ($entries -notcontains $bin) {
    [Environment]::SetEnvironmentVariable('Path', (@($bin) + $entries | Where-Object { $_ }) -join ';', 'User')
}
$env:Path = "$bin;$env:Path"

$version = (& (Join-Path $bin 'ember.exe') --version | Select-Object -First 1)
Write-Host "installed $version in $home_dir"
Write-Host 'open a new terminal, then: ember new hello; cd hello; ember run'
