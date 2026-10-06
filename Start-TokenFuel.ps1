# Prebuilt app launcher. No Rust, Node.js, Python, admin rights, or provider secrets required.
[CmdletBinding()]
param([switch]$DownloadOnly)
$ErrorActionPreference = 'Stop'
function Get-FuelHash([string]$Path) {
    $fuelHasher = [Security.Cryptography.SHA256]::Create()
    $fuelStream = [IO.File]::OpenRead($Path)
    try { return ([BitConverter]::ToString($fuelHasher.ComputeHash($fuelStream))).Replace('-','') }
    finally { $fuelStream.Dispose(); $fuelHasher.Dispose() }
}
$fuelVersion = 'v0.2.0'
# A rebuilt release needs its own cache so an older 0.2.0 executable is not reused.
$fuelRevision = '2026-10-06.3'
$fuelRepo = 'tejas20/TokenFuel'
$fuelZip = 'TokenFuel_0.2.0_x64-portable.zip'
if ($env:OS -ne 'Windows_NT' -or -not [Environment]::Is64BitOperatingSystem) {
    throw 'This preview requires Windows 10/11 x64.'
}
$fuelInstall = Join-Path $env:LOCALAPPDATA "TokenFuel/Preview/$fuelVersion/$fuelRevision"
$fuelExe = Join-Path $fuelInstall 'TokenFuel.exe'
$fuelManifest = Join-Path $fuelInstall 'executable.sha256'
$fuelCached = (Test-Path -LiteralPath $fuelExe) -and (Test-Path -LiteralPath $fuelManifest)
if ($fuelCached) {
    $fuelCached = (Get-FuelHash $fuelExe) -eq (Get-Content -LiteralPath $fuelManifest -Raw).Trim()
}
if (-not $fuelCached) {
    $fuelStage = Join-Path ([IO.Path]::GetTempPath()) ('TokenFuel-download-' + [Guid]::NewGuid())
    New-Item -ItemType Directory -Path $fuelStage | Out-Null
    try {
        Write-Host 'Downloading public TokenFuel release...'
        [Net.ServicePointManager]::SecurityProtocol = [Net.ServicePointManager]::SecurityProtocol -bor [Net.SecurityProtocolType]::Tls12
        $fuelReleaseUrl = "https://github.com/$fuelRepo/releases/download/$fuelVersion"
        foreach ($fuelAsset in @($fuelZip, 'SHA256SUMS.txt')) {
            Invoke-WebRequest -UseBasicParsing -Uri "$fuelReleaseUrl/$fuelAsset" -OutFile (Join-Path $fuelStage $fuelAsset)
        }
        $fuelPattern = '^([0-9a-fA-F]{64})\s+\*?' + [Regex]::Escape($fuelZip) + '$'
        $fuelChecksums = @(Get-Content -LiteralPath (Join-Path $fuelStage 'SHA256SUMS.txt') | Where-Object { $_ -match $fuelPattern })
        if ($fuelChecksums.Count -ne 1) { throw 'The release checksum is missing or ambiguous; nothing was launched.' }
        $fuelExpected = [Regex]::Match($fuelChecksums[0], $fuelPattern).Groups[1].Value
        $fuelArchive = Join-Path $fuelStage $fuelZip
        if ((Get-FuelHash $fuelArchive) -ne $fuelExpected) { throw 'Download checksum mismatch; nothing was launched.' }
        Add-Type -AssemblyName System.IO.Compression.FileSystem
        $fuelEntries = [IO.Compression.ZipFile]::OpenRead($fuelArchive)
        try {
            foreach ($fuelEntry in $fuelEntries.Entries) {
                if ([IO.Path]::IsPathRooted($fuelEntry.FullName) -or $fuelEntry.FullName -match '(^|[/\\])\.\.([/\\]|$)' -or $fuelEntry.FullName.Contains(':')) { throw 'Unsafe archive path; nothing was extracted.' }
            }
        } finally { $fuelEntries.Dispose() }
        New-Item -ItemType Directory -Force -Path $fuelInstall | Out-Null
        Expand-Archive -LiteralPath $fuelArchive -DestinationPath $fuelInstall -Force
        if (-not (Test-Path -LiteralPath $fuelExe)) { throw 'The release did not contain TokenFuel.exe.' }
        Get-FuelHash $fuelExe | Set-Content -LiteralPath $fuelManifest
    } finally {
        # Only remove this invocation's GUID-named temporary directory, after validating its boundary.
        $fuelResolved = [IO.Path]::GetFullPath($fuelStage)
        $fuelTempRoot = [IO.Path]::GetFullPath([IO.Path]::GetTempPath()).TrimEnd('\','/') + [IO.Path]::DirectorySeparatorChar
        if ($fuelResolved.StartsWith($fuelTempRoot, [StringComparison]::OrdinalIgnoreCase) -and [IO.Path]::GetFileName($fuelResolved).StartsWith('TokenFuel-download-')) {
            Remove-Item -LiteralPath $fuelResolved -Recurse -Force
        }
    }
}
if ($DownloadOnly) { Write-Output $fuelExe; return }
foreach ($fuelExisting in [Diagnostics.Process]::GetProcessesByName('TokenFuel')) {
    try {
        if ($fuelExisting.MainModule.FileName -eq $fuelExe) {
            Write-Host 'TokenFuel is already running. Use its tray menu to show the widget.'
            return
        }
    } catch { } finally { $fuelExisting.Dispose() }
}
Write-Host 'Starting TokenFuel. Review connections in Settings; startup remains opt-in.'
Start-Process -FilePath $fuelExe -WindowStyle Hidden
