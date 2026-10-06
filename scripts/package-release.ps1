# Build only. Publishing is a separate maintainer action.
[CmdletBinding()]
param([switch]$SkipBuild)
$ErrorActionPreference = 'Stop'
$fuelRoot = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
Push-Location $fuelRoot
try {
    $fuelVersion = (Get-Content package.json -Raw | ConvertFrom-Json).version
    $fuelOut = Join-Path $fuelRoot "artifacts/release/v$fuelVersion"
    New-Item -ItemType Directory -Force -Path $fuelOut | Out-Null
    if (-not $SkipBuild) {
        & pnpm tauri build --ci
        if ($LASTEXITCODE -ne 0) { throw 'Release build failed.' }
    }
    $fuelInstaller = Join-Path $fuelRoot "target/release/bundle/nsis/TokenFuel_${fuelVersion}_x64-setup.exe"
    if (-not (Test-Path -LiteralPath $fuelInstaller)) { throw 'Regular installer is missing.' }
    Copy-Item -LiteralPath $fuelInstaller -Destination $fuelOut -Force
    & pnpm tauri bundle --ci --config src-tauri/tauri.offline.conf.json --bundles nsis
    if ($LASTEXITCODE -ne 0) { throw 'Offline installer packaging failed.' }
    Copy-Item -LiteralPath $fuelInstaller -Destination (Join-Path $fuelOut "TokenFuel_${fuelVersion}_x64-offline-setup.exe") -Force
    # Leave the default build output pointing to the regular installer.
    Copy-Item -LiteralPath (Join-Path $fuelOut "TokenFuel_${fuelVersion}_x64-setup.exe") -Destination $fuelInstaller -Force
    $fuelStage = Join-Path $fuelOut 'portable'
    New-Item -ItemType Directory -Force -Path $fuelStage | Out-Null
    Copy-Item -LiteralPath (Join-Path $fuelRoot 'target/release/tokenfuel.exe') -Destination (Join-Path $fuelStage 'TokenFuel.exe') -Force
    Copy-Item -LiteralPath LICENSE,THIRD_PARTY_NOTICES.md -Destination $fuelStage -Force
    Copy-Item -LiteralPath public/providers/LICENSE-octicons,public/providers/LICENSE-lobe-icons,docs/design/LICENSE-phosphor -Destination $fuelStage -Force
    Copy-Item -LiteralPath docs/portable-readme.txt -Destination (Join-Path $fuelStage 'README.txt') -Force
    $fuelZip = Join-Path $fuelOut "TokenFuel_${fuelVersion}_x64-portable.zip"
    Compress-Archive -Path (Join-Path $fuelStage '*') -DestinationPath $fuelZip -Force
    $fuelSums = foreach ($fuelFile in (Get-ChildItem -LiteralPath $fuelOut -File | Where-Object { $_.Extension -in '.exe','.zip' } | Sort-Object Name)) {
        $fuelHasher = [Security.Cryptography.SHA256]::Create()
        $fuelStream = [IO.File]::OpenRead($fuelFile.FullName)
        try { $fuelHash = ([BitConverter]::ToString($fuelHasher.ComputeHash($fuelStream))).Replace('-','').ToLowerInvariant() }
        finally { $fuelStream.Dispose(); $fuelHasher.Dispose() }
        "$fuelHash  $($fuelFile.Name)"
    }
    $fuelSums | Set-Content -LiteralPath (Join-Path $fuelOut 'SHA256SUMS.txt') -Encoding ascii
    Write-Output $fuelOut
} finally { Pop-Location }
