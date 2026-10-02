# Prebuilt app launcher. No Rust, Node.js, Python, admin rights, or provider secrets required.
[CmdletBinding()]
param([switch]$DownloadOnly)
$ErrorActionPreference = 'Stop'
$fuelVersion = 'v0.1.0-review.1'
$fuelRepo = 'tejas20/TokenFuel'
$fuelZip = 'TokenFuel_0.1.0_x64-portable.zip'
if ($env:OS -ne 'Windows_NT' -or -not [Environment]::Is64BitOperatingSystem) {
    throw 'This preview requires Windows 10/11 x64.'
}
$fuelInstall = Join-Path $env:LOCALAPPDATA "TokenFuel/Preview/$fuelVersion"
$fuelExe = Join-Path $fuelInstall 'TokenFuel.exe'
$fuelManifest = Join-Path $fuelInstall 'executable.sha256'
$fuelCached = (Test-Path -LiteralPath $fuelExe) -and (Test-Path -LiteralPath $fuelManifest)
if ($fuelCached) {
    $fuelCached = (Get-FileHash -LiteralPath $fuelExe -Algorithm SHA256).Hash -eq (Get-Content -LiteralPath $fuelManifest -Raw).Trim()
}
if (-not $fuelCached) {
    $fuelGhCommand = Get-Command gh -ErrorAction SilentlyContinue
    $fuelGh = if ($fuelGhCommand) { $fuelGhCommand.Source } else { Join-Path $env:ProgramFiles 'GitHub CLI/gh.exe' }
    if (-not (Test-Path -LiteralPath $fuelGh)) {
        throw 'Private preview: install GitHub CLI (winget install GitHub.cli), run gh auth login, then retry. No developer toolchain is needed.'
    }
    $fuelStage = Join-Path ([IO.Path]::GetTempPath()) ('TokenFuel-download-' + [Guid]::NewGuid())
    New-Item -ItemType Directory -Path $fuelStage | Out-Null
    try {
        Write-Host 'Downloading private TokenFuel preview...'
        & $fuelGh release download $fuelVersion --repo $fuelRepo --pattern $fuelZip --pattern 'SHA256SUMS.txt' --dir $fuelStage
        if ($LASTEXITCODE -ne 0) { throw 'Download failed. Run gh auth login with an account that can access tejas20/TokenFuel.' }
        $fuelPattern = '^([0-9a-fA-F]{64})\s+\*?' + [Regex]::Escape($fuelZip) + '$'
        $fuelChecksums = @(Get-Content -LiteralPath (Join-Path $fuelStage 'SHA256SUMS.txt') | Where-Object { $_ -match $fuelPattern })
        if ($fuelChecksums.Count -ne 1) { throw 'The release checksum is missing or ambiguous; nothing was launched.' }
        $fuelExpected = [Regex]::Match($fuelChecksums[0], $fuelPattern).Groups[1].Value
        $fuelArchive = Join-Path $fuelStage $fuelZip
        if ((Get-FileHash -LiteralPath $fuelArchive -Algorithm SHA256).Hash -ne $fuelExpected) { throw 'Download checksum mismatch; nothing was launched.' }
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
        (Get-FileHash -LiteralPath $fuelExe -Algorithm SHA256).Hash | Set-Content -LiteralPath $fuelManifest
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
Write-Host 'Starting TokenFuel. Connect an account in Settings; startup remains opt-in.'
Start-Process -FilePath $fuelExe
