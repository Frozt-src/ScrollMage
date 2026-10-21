#requires -Version 7.0
[CmdletBinding()]
param([switch]$Offline)
$ErrorActionPreference = 'Stop'
$repo = Split-Path $PSScriptRoot -Parent
$priorTarget = $env:CARGO_TARGET_DIR
$priorFlags = $env:RUSTFLAGS
$priorEncodedFlags = $env:CARGO_ENCODED_RUSTFLAGS
Push-Location $repo
try {
    $manifest = Get-Content -LiteralPath 'source/Cargo.toml' -Raw
    if ($manifest -notmatch '(?m)^version = "(\d+\.\d+\.\d+)"') { throw 'Missing package version.' }
    $version = $Matches[1]
    [xml]$appManifest = Get-Content -LiteralPath 'source/app.manifest' -Raw
    if ($appManifest.assembly.assemblyIdentity.version -ne "$version.0") { throw 'Manifest and Cargo versions disagree.' }
    $name = "ScrollMage-$version-candidate-windows-x64"
    $staging = Join-Path $repo "dist/$name"
    $zip = Join-Path $repo "dist/$name.zip"
    if ((Test-Path -LiteralPath $staging) -or (Test-Path -LiteralPath $zip)) { throw 'Candidate already exists; preserve and inspect it before creating another.' }
    $env:CARGO_TARGET_DIR = Join-Path $repo 'source/target'
    $cargoHomePath = if ($env:CARGO_HOME) { $env:CARGO_HOME } else { Join-Path $env:USERPROFILE '.cargo' }
    $env:CARGO_ENCODED_RUSTFLAGS = @('-C','target-feature=+crt-static',"--remap-path-prefix=$repo=/build","--remap-path-prefix=$cargoHomePath=/cargo") -join [char]31
    $env:RUSTFLAGS = $null
    $buildArgs = @('build','--manifest-path','source/Cargo.toml','--locked','--release','--target','x86_64-pc-windows-msvc')
    if ($Offline) { $buildArgs += '--offline' }
    & cargo @buildArgs
    if ($LASTEXITCODE -ne 0) { throw 'Release build failed.' }
    $metaArgs = @('metadata','--manifest-path','source/Cargo.toml','--locked','--format-version','1')
    if ($Offline) { $metaArgs += '--offline' }
    $metadataText = & cargo @metaArgs
    if ($LASTEXITCODE -ne 0) { throw 'Dependency inventory failed.' }
    $metadata = $metadataText | ConvertFrom-Json
    $dependencies = @($metadata.packages | Where-Object { $_.source } | Sort-Object name | ForEach-Object {
        [ordered]@{name=$_.name;version=$_.version;license=$_.license;source=$_.source}
    })
    $package = Join-Path $staging 'ScrollMage'
    New-Item -ItemType Directory -Path (Join-Path $package 'assets') -Force | Out-Null
    Copy-Item -LiteralPath 'source/target/x86_64-pc-windows-msvc/release/scrollpilot.exe' -Destination (Join-Path $package 'ScrollMage.exe')
    foreach ($file in @('ScrollMage.ico','LICENSE','THIRD_PARTY_NOTICES.md','CHANGELOG.md')) { Copy-Item -LiteralPath $file -Destination $package }
    Copy-Item -LiteralPath 'source/docs/MOUSE-ACCEPTANCE-0.6.10.md' -Destination (Join-Path $package 'MOUSE ACCEPTANCE.md')
    Copy-Item -LiteralPath 'assets/scroll-mage-banner.png' -Destination (Join-Path $package 'assets')
    if (@(Get-ChildItem -LiteralPath 'source/vendor-licenses' -Recurse -Force | Where-Object { $_.Attributes -band [IO.FileAttributes]::ReparsePoint }).Count) { throw 'License tree contains an unexpected link.' }
    Copy-Item -LiteralPath 'source/vendor-licenses' -Destination (Join-Path $package 'vendor-licenses') -Recurse
    $utf8 = [Text.UTF8Encoding]::new($false)
    $readme = [IO.File]::ReadAllText((Join-Path $repo 'README.md')).Replace('Start Paused.lnk','Start Paused.cmd')
    $readme = $readme -replace '(?m)^The local \*\*0\.6\.10 candidate\*\*.*$', 'This is an unsigned 0.6.10 candidate. Physical-device, accessibility and visible shell acceptance remain incomplete.'
    [IO.File]::WriteAllText((Join-Path $package 'README.md'),$readme,$utf8)
    [IO.File]::WriteAllText((Join-Path $package 'Start Paused.cmd'),"@echo off`r`nstart `"`" /D `"%~dp0`" `"%~dp0ScrollMage.exe`" --paused`r`n",$utf8)
    $instructions = @"
SCROLL MAGE $version CANDIDATE - WINDOWS X64

Extract this entire folder to a writable location. Quit any existing Scroll Mage
instance from the tray menu, then run ScrollMage.exe. Select Super smooth for
flick-and-coast scrolling. Reverse, click, or press Alt+P to stop coasting.
Browsers are excluded by default; remove your browser from Excluded apps to try it.

No personal settings are bundled. This candidate is unsigned and is not a claim
of completed physical-mouse, app-compatibility, accessibility or Snap acceptance.
Keep the included license notices with redistributed copies. See README.md and
MOUSE ACCEPTANCE.md for the remaining hands-on checks and rollback instructions.
"@
    [IO.File]::WriteAllText((Join-Path $package 'START HERE.txt'),$instructions+"`n",$utf8)
    $inventory = [ordered]@{version=$version;target='x86_64-pc-windows-msvc';lock_sha256=(Get-FileHash -LiteralPath 'source/Cargo.lock').Hash.ToLowerInvariant();dependencies=$dependencies}
    [IO.File]::WriteAllText((Join-Path $package 'DEPENDENCIES.json'),($inventory | ConvertTo-Json -Depth 6)+"`n",$utf8)
    $files = @(Get-ChildItem -LiteralPath $package -Recurse -File | Sort-Object FullName)
    $hashes = $files | ForEach-Object { '{0}  {1}' -f (Get-FileHash -LiteralPath $_.FullName).Hash.ToLowerInvariant(), [IO.Path]::GetRelativePath($package,$_.FullName).Replace('\','/') }
    [IO.File]::WriteAllLines((Join-Path $package 'SHA256SUMS.txt'),$hashes,$utf8)
    Compress-Archive -LiteralPath $package -DestinationPath $zip -CompressionLevel Optimal
    $exeHash = (Get-FileHash -LiteralPath (Join-Path $package 'ScrollMage.exe')).Hash
    & (Join-Path $PSScriptRoot 'Verify-Package.ps1') -Path $zip -ExpectedExecutableHash $exeHash
    $zipHash = (Get-FileHash -LiteralPath $zip).Hash.ToLowerInvariant()
    [IO.File]::WriteAllText(($zip+'.sha256'),$zipHash+'  '+[IO.Path]::GetFileName($zip)+"`n",$utf8)
    Write-Output "Candidate: $zip"
    Write-Output "SHA256: $zipHash"
} finally {
    $env:CARGO_TARGET_DIR = $priorTarget
    $env:RUSTFLAGS = $priorFlags
    $env:CARGO_ENCODED_RUSTFLAGS = $priorEncodedFlags
    Pop-Location
}
