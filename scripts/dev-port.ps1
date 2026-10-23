<#
.SYNOPSIS
  Pick a free dev-server port and keep vite.config.ts / tauri.conf.json in sync.

.DESCRIPTION
  Windows reserves port ranges for Hyper-V / WinNAT, and `tauri dev` fails when
  the dev port lands inside one. This script:

    1. reads reserved ranges (`netsh int ipv4 show excludedportrange protocol=tcp`)
       and the dynamic range (`netsh int ipv4 show dynamicport tcp`),
    2. walks a candidate list until a port (and its HMR neighbour) is free and
       actually bindable on loopback,
    3. rewrites `const DEV_PORT = <n>;` in apps/windows/vite.config.ts and
       `"devUrl": "http://localhost:<n>"` in apps/windows/src-tauri/tauri.conf.json.

  Both files are tracked, so `git checkout -- apps/windows/vite.config.ts
  apps/windows/src-tauri/tauri.conf.json` reverts the change.

.PARAMETER Port
  Force a specific port instead of scanning. Fails if it is reserved or busy.

.PARAMETER DryRun
  Only report the port that would be used; do not touch any file.

.EXAMPLE
  powershell -ExecutionPolicy Bypass -File scripts/dev-port.ps1

.EXAMPLE
  powershell -ExecutionPolicy Bypass -File scripts/dev-port.ps1 -Port 1620
#>
[CmdletBinding()]
param(
    [int]$Port,
    [switch]$DryRun
)

$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$viteConfig = Join-Path $repoRoot "apps/windows/vite.config.ts"
$tauriConfig = Join-Path $repoRoot "apps/windows/src-tauri/tauri.conf.json"

foreach ($path in @($viteConfig, $tauriConfig)) {
    if (-not (Test-Path $path)) {
        throw "Expected file not found: $path. Run this script from the repository."
    }
}

$candidates = @(
    1520, 1530, 1540, 1560, 1580, 1600, 1620, 1640, 1660, 1680,
    1700, 1800, 1900, 2000, 2100, 2200, 2480, 2800, 3000, 3300,
    3600, 4200, 4800, 5173, 8080, 9090
)

function Get-ExcludedRanges {
    $ranges = @()
    $out = & netsh int ipv4 show excludedportrange protocol=tcp 2>$null
    foreach ($line in $out) {
        if ($line -match '^\s*(\d{2,5})\s+(\d{2,5})\s*$') {
            $ranges += [pscustomobject]@{
                Start = [int]$Matches[1]
                End   = [int]$Matches[2]
            }
        }
    }
    return $ranges
}

function Get-DynamicRange {
    $out = & netsh int ipv4 show dynamicport tcp 2>$null
    $numbers = @()
    foreach ($line in $out) {
        # "Start Port : 49152" / "Number of Ports : 16384" (labels are localised)
        if ($line -match ':\s*(\d+)') { $numbers += [int]$Matches[1] }
    }
    if ($numbers.Count -ge 2) {
        return [pscustomobject]@{
            Start = $numbers[0]
            End   = $numbers[0] + $numbers[1] - 1
        }
    }
    return $null
}

function Test-InRange([int]$value, $range) {
    return $range -and $value -ge $range.Start -and $value -le $range.End
}

function Test-Bindable([int]$value) {
    $listener = $null
    try {
        $listener = [System.Net.Sockets.TcpListener]::new([System.Net.IPAddress]::Loopback, $value)
        $listener.Start()
        return $true
    } catch {
        return $false
    } finally {
        if ($listener) { $listener.Stop() }
    }
}

$excluded = Get-ExcludedRanges
$dynamic = Get-DynamicRange

Write-Host "Reserved (Hyper-V / WinNAT):"
if ($excluded.Count -eq 0) {
    Write-Host "  none reported"
} else {
    foreach ($range in $excluded) { Write-Host ("  {0}-{1}" -f $range.Start, $range.End) }
}
if ($dynamic) {
    Write-Host ("Dynamic range: {0}-{1}" -f $dynamic.Start, $dynamic.End)
}

function Test-PortUsable([int]$value) {
    if (Test-InRange $value $dynamic) { return $false }
    foreach ($range in $excluded) {
        if (Test-InRange $value $range) { return $false }
    }
    return (Test-Bindable $value)
}

if ($Port -gt 0) {
    if (-not (Test-PortUsable $Port)) {
        throw ("Port {0} is reserved by the system or already in use. Re-run without -Port to scan." -f $Port)
    }
    $chosen = $Port
} else {
    $chosen = $null
    foreach ($candidate in $candidates) {
        if ((Test-PortUsable $candidate) -and (Test-PortUsable ($candidate + 1))) {
            $chosen = $candidate
            break
        }
    }
    if (-not $chosen) {
        throw "No free port found in the candidate list. Add a port to `$candidates and retry."
    }
}

Write-Host ("Chosen dev port: {0} (HMR {1})" -f $chosen, ($chosen + 1))

if ($DryRun) {
    Write-Host "DryRun: files left untouched."
    return
}

$viteText = [System.IO.File]::ReadAllText($viteConfig)
$updatedVite = [regex]::Replace($viteText, 'const DEV_PORT = \d+;', "const DEV_PORT = $chosen;")
if ($updatedVite -eq $viteText) {
    throw "Could not find 'const DEV_PORT = <n>;' in $viteConfig"
}
# UTF8 without BOM: a BOM breaks serde_json when Tauri reads tauri.conf.json.
$utf8NoBom = New-Object System.Text.UTF8Encoding($false)
[System.IO.File]::WriteAllText($viteConfig, $updatedVite, $utf8NoBom)

$tauriText = [System.IO.File]::ReadAllText($tauriConfig)
$updatedTauri = [regex]::Replace(
    $tauriText,
    '"devUrl":\s*"http://localhost:\d+"',
    ('"devUrl": "http://localhost:{0}"' -f $chosen)
)
if ($updatedTauri -eq $tauriText) {
    throw "Could not find a devUrl entry in $tauriConfig"
}
[System.IO.File]::WriteAllText($tauriConfig, $updatedTauri, $utf8NoBom)

Write-Host ""
Write-Host "Updated:"
Write-Host ("  apps/windows/vite.config.ts          -> const DEV_PORT = {0};" -f $chosen)
Write-Host ("  apps/windows/src-tauri/tauri.conf.json -> http://localhost:{0}" -f $chosen)
Write-Host ""
Write-Host "Next: npm --prefix apps/windows run tauri dev"
Write-Host "Revert: git checkout -- apps/windows/vite.config.ts apps/windows/src-tauri/tauri.conf.json"
