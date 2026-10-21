#requires -Version 7.0
[CmdletBinding()]
param([switch]$Offline, [switch]$DesktopInput)
$ErrorActionPreference = 'Stop'
$repo = Split-Path $PSScriptRoot -Parent
$source = Join-Path $repo 'source'
$priorTarget = $env:CARGO_TARGET_DIR
Push-Location $source
try {
    $env:CARGO_TARGET_DIR = Join-Path $source 'target'
    $locked = @('--locked')
    if ($Offline) { $locked += '--offline' }
    function Invoke-Cargo([string[]]$Arguments) {
        & cargo @Arguments
        if ($LASTEXITCODE -ne 0) { throw "Cargo failed ($LASTEXITCODE): $($Arguments -join ' ')" }
    }
    Invoke-Cargo @('fmt', '--all', '--', '--check')
    Invoke-Cargo (@('clippy') + $locked + @('--all-targets', '--', '-D', 'warnings'))
    $tests = @('test') + $locked + @('--release', '--target', 'x86_64-pc-windows-msvc', '--lib')
    foreach ($test in Get-ChildItem -LiteralPath (Join-Path $source 'tests') -Filter '*.rs' -File | Sort-Object Name) {
        $tests += @('--test', $test.BaseName)
    }
    Invoke-Cargo $tests
    Invoke-Cargo (@('test') + $locked + @('--release', '--target', 'x86_64-pc-windows-msvc', '--bin', 'scrollpilot', 'hidden_native_', '--', '--nocapture', '--test-threads=1'))
    if ($DesktopInput) {
        Invoke-Cargo (@('test') + $locked + @('--release', '--target', 'x86_64-pc-windows-msvc', '--bin', 'scrollpilot', 'owned_window_receives_smoothed_tagged_input_and_worker_stops_when_idle', '--', '--nocapture', '--test-threads=1'))
    }
    Write-Output 'Validation passed. Physical-device and visible shell acceptance are separate checks.'
} finally {
    $env:CARGO_TARGET_DIR = $priorTarget
    Pop-Location
}
