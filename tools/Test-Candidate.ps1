#requires -Version 7.0
[CmdletBinding()]
param([Parameter(Mandatory)][string]$Executable)
$ErrorActionPreference = 'Stop'
$repo = Split-Path $PSScriptRoot -Parent
$exe = (Resolve-Path -LiteralPath $Executable).Path
if (Get-Process -Name ScrollMage,scrollpilot -ErrorAction SilentlyContinue) {
    throw 'Quit existing Scroll Mage instances before the isolated candidate check.'
}
if (-not ('ScrollMageCandidateProbe' -as [type])) {
    Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class ScrollMageCandidateProbe {
    public delegate bool EnumProc(IntPtr window, IntPtr parameter);
    [DllImport("user32.dll")] public static extern bool EnumWindows(EnumProc callback, IntPtr parameter);
    [DllImport("user32.dll")] public static extern bool EnumChildWindows(IntPtr parent, EnumProc callback, IntPtr parameter);
    [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr window, out uint process);
    [DllImport("user32.dll")] public static extern int GetDlgCtrlID(IntPtr window);
    [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern int GetWindowText(IntPtr window, System.Text.StringBuilder text, int count);
    [DllImport("user32.dll")] public static extern IntPtr SendMessageTimeout(IntPtr window, uint message, IntPtr w, IntPtr l, uint flags, uint timeout, out UIntPtr result);
    [DllImport("user32.dll")] public static extern uint GetGuiResources(IntPtr process, uint flag);
}
'@
}
function Send-ProbeMessage([IntPtr]$Window, [uint32]$Message, [long]$W = 0, [long]$L = 0) {
    [UIntPtr]$result = [UIntPtr]::Zero
    if ([ScrollMageCandidateProbe]::SendMessageTimeout($Window,$Message,[IntPtr]$W,[IntPtr]$L,2,3000,[ref]$result) -eq [IntPtr]::Zero) {
        throw "Candidate message timed out or failed: $Message"
    }
}
function Find-ProbeControl([IntPtr]$Window, [int]$Id) {
    $found = [Collections.Generic.List[IntPtr]]::new()
    [ScrollMageCandidateProbe]::EnumChildWindows($Window,{
        param($child,$parameter)
        if ([ScrollMageCandidateProbe]::GetDlgCtrlID($child) -eq $Id) { $found.Add($child) }
        return $true
    },[IntPtr]::Zero) | Out-Null
    if ($found.Count -ne 1) { throw "Expected one control $Id; found $($found.Count)." }
    return $found[0]
}
function Read-ProbeText([IntPtr]$Window) {
    $text = [Text.StringBuilder]::new(4096)
    [ScrollMageCandidateProbe]::GetWindowText($Window,$text,$text.Capacity) | Out-Null
    return $text.ToString()
}
$fixture = Join-Path $repo ('source/target/candidate-smoke/Portable space-工具-'+[Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $fixture -Force | Out-Null
$copy = Join-Path $fixture 'ScrollMage.exe'
Copy-Item -LiteralPath $exe -Destination $copy
$settings = Join-Path $fixture 'settings.json'
$results = @()
foreach ($scenario in @('fresh','corrupt','blocked')) {
    if ($scenario -eq 'corrupt') { [IO.File]::WriteAllText($settings,'{broken') }
    if ($scenario -eq 'blocked') {
        Remove-Item -LiteralPath $settings
        New-Item -ItemType Directory -Path $settings | Out-Null
    }
    $process = $null
    try {
        # Only this owned copy is launched; no global settings, sign-in values or mouse input change.
        $process = Start-Process -FilePath $copy -ArgumentList '--paused','--tray' -WorkingDirectory $repo -WindowStyle Hidden -PassThru
        $windows = [Collections.Generic.List[IntPtr]]::new()
        $deadline = [DateTime]::UtcNow.AddSeconds(10)
        do {
            $windows.Clear()
            [ScrollMageCandidateProbe]::EnumWindows({
                param($window,$parameter)
                [uint32]$owner = 0
                [ScrollMageCandidateProbe]::GetWindowThreadProcessId($window,[ref]$owner) | Out-Null
                if ($owner -eq $process.Id -and (Read-ProbeText $window) -eq 'Scroll Mage') { $windows.Add($window) }
                return $true
            },[IntPtr]::Zero) | Out-Null
            if ($windows.Count -eq 1) { break }
            if ($process.HasExited) { throw "Candidate exited before creating its window ($scenario)." }
            Start-Sleep -Milliseconds 50
        } while ([DateTime]::UtcNow -lt $deadline)
        if ($windows.Count -ne 1) { throw "Candidate window was not found ($scenario)." }
        if (-not $process.WaitForInputIdle(5000)) { throw 'Candidate did not finish creating its controls.' }
        $window = $windows[0]
        $toggle = Find-ProbeControl $window 102
        if ((Read-ProbeText $toggle) -ne 'Enable smoothing') { throw 'Candidate must start paused.' }
        if ($scenario -eq 'fresh' -and (Test-Path -LiteralPath $settings)) { throw 'Startup unexpectedly wrote settings.' }
        if ($scenario -eq 'corrupt' -and [IO.File]::ReadAllText($settings) -ne '{broken') { throw 'Loading overwrote corrupt settings.' }
        $preset = Find-ProbeControl $window 135
        Send-ProbeMessage $preset 0x14e 2 # CB_SETCURSEL: Super smooth
        Send-ProbeMessage $window 0x111 ((1 -shl 16) -bor 135) $preset.ToInt64() # CBN_SELCHANGE
        if ($scenario -eq 'fresh') {
            $saved = Get-Content -LiteralPath $settings -Raw | ConvertFrom-Json
            if ($saved.preset -ne 'super_smooth') { throw 'Preset did not save beside the portable executable.' }
            $process.Refresh()
            $cpu = $process.TotalProcessorTime.TotalMilliseconds
            $handles = $process.HandleCount
            $gdi = [ScrollMageCandidateProbe]::GetGuiResources($process.Handle,0)
            $user = [ScrollMageCandidateProbe]::GetGuiResources($process.Handle,1)
            Start-Sleep -Seconds 3
            $process.Refresh()
            $results += [ordered]@{scenario=$scenario;passed=$true;idle_seconds=3;cpu_ms=$process.TotalProcessorTime.TotalMilliseconds-$cpu;handle_delta=$process.HandleCount-$handles;gdi_delta=[int][ScrollMageCandidateProbe]::GetGuiResources($process.Handle,0)-[int]$gdi;user_delta=[int][ScrollMageCandidateProbe]::GetGuiResources($process.Handle,1)-[int]$user}
        } elseif ($scenario -eq 'corrupt') {
            # A deliberate successful edit replaces corrupt settings; loading alone must preserve them.
            if ((Get-Content -LiteralPath $settings -Raw | ConvertFrom-Json).preset -ne 'super_smooth') { throw 'Recovery edit did not save.' }
            $results += [ordered]@{scenario=$scenario;passed=$true}
        } else {
            if (-not (Test-Path -LiteralPath $settings -PathType Container)) { throw 'Failed save replaced the blocked target.' }
            # Query the selection: failed auto-save must roll back the visible preset.
            [UIntPtr]$selection = [UIntPtr]::Zero
            [ScrollMageCandidateProbe]::SendMessageTimeout($preset,0x147,[IntPtr]::Zero,[IntPtr]::Zero,2,3000,[ref]$selection) | Out-Null
            if ($selection.ToUInt64() -ne 0) { throw 'Failed save did not restore the active Default preset.' }
            $results += [ordered]@{scenario=$scenario;passed=$true}
        }
        Send-ProbeMessage $window 0x111 104 # Quit command, not close-to-tray.
        if (-not $process.WaitForExit(5000) -or $process.ExitCode -ne 0) { throw 'Candidate did not shut down cleanly.' }
    } finally {
        if ($process -and -not $process.HasExited) { Stop-Process -Id $process.Id }
        if ($process) { $process.Dispose() }
    }
}
$report = [ordered]@{checked_at_utc=[DateTime]::UtcNow.ToString('o');exe_sha256=(Get-FileHash -LiteralPath $copy).Hash.ToLowerInvariant();fixture=$fixture;checks=$results;limits='Paused packaged binary only; no physical wheel, Narrator, sign-in or Explorer restart acceptance.'}
$reportPath = Join-Path $repo 'source/target/candidate-smoke.json'
$report | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $reportPath -Encoding utf8
Write-Output "Passed $($results.Count) packaged candidate scenarios. Evidence: $reportPath"
