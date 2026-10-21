#requires -Version 7.0
[CmdletBinding()]
param([Parameter(Mandatory)][string]$Path)
$ErrorActionPreference = 'Stop'
# Establish a valid baseline: otherwise a verifier that rejects everything could
# falsely pass every rejection scenario below.
& (Join-Path $PSScriptRoot 'Verify-Package.ps1') -Path $Path
$repo = Split-Path $PSScriptRoot -Parent
$fixtures = Join-Path $repo 'source/target/package-verifier-tests'
New-Item -ItemType Directory -Path $fixtures -Force | Out-Null
Add-Type -AssemblyName System.IO.Compression.FileSystem
$passed = 0
foreach ($scenario in @('tampered-readme','personal-settings','path-traversal')) {
    $fixture = Join-Path $fixtures ($scenario+'-'+[Guid]::NewGuid().ToString('N')+'.zip')
    try {
        Copy-Item -LiteralPath $Path -Destination $fixture
        $archive = [IO.Compression.ZipFile]::Open($fixture,[IO.Compression.ZipArchiveMode]::Update)
        try {
            $entryName = switch ($scenario) {
                'tampered-readme' { $archive.GetEntry('ScrollMage/README.md').Delete(); 'ScrollMage/README.md' }
                'personal-settings' { 'ScrollMage/settings.json' }
                'path-traversal' { 'ScrollMage/../escape.txt' }
            }
            $writer = [IO.StreamWriter]::new($archive.CreateEntry($entryName).Open())
            try { $writer.Write('deliberately invalid test fixture') } finally { $writer.Dispose() }
        } finally { $archive.Dispose() }
        $rejected = $false
        try { & (Join-Path $PSScriptRoot 'Verify-Package.ps1') -Path $fixture | Out-Null }
        catch { $rejected = $true; Write-Output "Correctly rejected $scenario`: $($_.Exception.Message)" }
        if (-not $rejected) { throw "Package verifier incorrectly accepted $scenario" }
        $passed++
    } finally {
        # Delete only the explicit fixture file created above; never recurse.
        if (Test-Path -LiteralPath $fixture) { Remove-Item -LiteralPath $fixture }
    }
}
Write-Output "Passed $passed adversarial package checks."
