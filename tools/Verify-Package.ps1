#requires -Version 7.0
[CmdletBinding()]
param([Parameter(Mandatory)][string]$Path, [string]$ExpectedExecutableHash)
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.IO.Compression.FileSystem
$archive = [IO.Compression.ZipFile]::OpenRead((Resolve-Path -LiteralPath $Path).Path)
try {
    $files = [Collections.Generic.Dictionary[string, object]]::new([StringComparer]::OrdinalIgnoreCase)
    [long]$bytes = 0
    foreach ($entry in $archive.Entries) {
        $name = $entry.FullName
        if ($name -notmatch '^ScrollMage/' -or $name.Contains('\') -or $name -match '(^|/)(\.|\.\.)(/|$)' -or (($entry.ExternalAttributes -shr 16) -band 0xf000) -eq 0xa000) {
            throw "Unsafe package path: $name"
        }
        if (-not $entry.Name) { continue }
        $relative = $name.Substring(11)
        $allowed = $relative -in @('ScrollMage.exe','ScrollMage.ico','README.md','CHANGELOG.md','MOUSE ACCEPTANCE.md','START HERE.txt','Start Paused.cmd','LICENSE','LICENSE-ARTWORK.md','THIRD_PARTY_NOTICES.md','DEPENDENCIES.json','SHA256SUMS.txt','assets/scroll-mage-banner.png') -or $relative -match '^vendor-licenses/[A-Za-z0-9_.-]+/(license|copying|copyright|notice)[A-Za-z0-9_.-]*$'
        if ($relative -match '\.(pem|pfx|key|rs|pdb)$') { $allowed = $false }
        if (-not $allowed -or $files.ContainsKey($relative)) { throw "Unexpected or duplicate package member: $relative" }
        $bytes += $entry.Length
        if ($entry.Length -gt 32MB -or $bytes -gt 64MB -or $files.Count -ge 256) { throw 'Package size/count budget exceeded.' }
        $files.Add($relative, $entry)
    }
    foreach ($required in @('ScrollMage.exe','ScrollMage.ico','README.md','CHANGELOG.md','MOUSE ACCEPTANCE.md','START HERE.txt','Start Paused.cmd','LICENSE','LICENSE-ARTWORK.md','THIRD_PARTY_NOTICES.md','DEPENDENCIES.json','SHA256SUMS.txt','assets/scroll-mage-banner.png')) {
        if (-not $files.ContainsKey($required)) { throw "Missing required package member: $required" }
    }
    if ($files['SHA256SUMS.txt'].Length -gt 65536) { throw 'Checksum list is too large.' }
    $reader = [IO.StreamReader]::new($files['SHA256SUMS.txt'].Open())
    try { $lines = $reader.ReadToEnd() -split '\r?\n' | Where-Object { $_ } } finally { $reader.Dispose() }
    $listed = [Collections.Generic.HashSet[string]]::new([StringComparer]::OrdinalIgnoreCase)
    foreach ($line in $lines) {
        if ($line -notmatch '^([0-9a-f]{64})  (.+)$') { throw 'Malformed checksum record.' }
        $expected = $Matches[1]; $relative = $Matches[2]
        if ($relative -eq 'SHA256SUMS.txt' -or -not $listed.Add($relative) -or -not $files.ContainsKey($relative)) { throw "Invalid checksum member: $relative" }
        $stream = $files[$relative].Open()
        try { $actual = [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($stream)).ToLowerInvariant() } finally { $stream.Dispose() }
        if ($actual -ne $expected) { throw "Hash mismatch: $relative" }
        if ($relative -eq 'ScrollMage.exe' -and $ExpectedExecutableHash -and $actual -ne $ExpectedExecutableHash.ToLowerInvariant()) { throw 'Packaged executable does not match the build.' }
    }
    if ($listed.Count -ne $files.Count - 1) { throw 'Unlisted package contents.' }
    Write-Output "Verified $($listed.Count) package hashes; only approved distributable paths are present."
} finally { $archive.Dispose() }
