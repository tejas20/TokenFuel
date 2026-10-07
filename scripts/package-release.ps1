# Build only. Publishing is a separate maintainer action.
[CmdletBinding()]
param(
    [switch]$SkipBuild,
    [switch]$AllowUnsigned,
    [string]$SigningConfig = $env:TOKENFUEL_SIGNING_CONFIG
)
$ErrorActionPreference = 'Stop'
if ($AllowUnsigned -and $SigningConfig) { throw 'Choose signed release packaging or explicit unsigned test packaging, not both.' }
if (-not $AllowUnsigned -and -not $SigningConfig) {
    throw 'Release packaging requires -SigningConfig (or TOKENFUEL_SIGNING_CONFIG). For local/CI test builds only, explicitly pass -AllowUnsigned. See docs/releasing.md.'
}
$fuelSigningArgs = @()
if ($SigningConfig) {
    $SigningConfig = (Resolve-Path -LiteralPath $SigningConfig).Path
    $fuelSigning = Get-Content -LiteralPath $SigningConfig -Raw | ConvertFrom-Json
    $fuelWindowsSigning = $fuelSigning.bundle.windows
    if (-not $fuelWindowsSigning.signCommand -and -not $fuelWindowsSigning.certificateThumbprint) {
        throw 'Signing config must configure bundle.windows.signCommand or certificateThumbprint.'
    }
    $fuelSigningArgs = @('--config', $SigningConfig)
}
$fuelRoot = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
Push-Location $fuelRoot
try {
    $fuelVersion = (Get-Content package.json -Raw | ConvertFrom-Json).version
    $fuelOut = Join-Path $fuelRoot "artifacts/release/v$fuelVersion"
    New-Item -ItemType Directory -Force -Path $fuelOut | Out-Null
    # A failed rebuild must not leave old attestations beside partially replaced assets.
    foreach ($fuelMetadata in @('SHA256SUMS.txt', 'SIGNATURES.json')) {
        $fuelMetadataPath = Join-Path $fuelOut $fuelMetadata
        if (Test-Path -LiteralPath $fuelMetadataPath) { Remove-Item -LiteralPath $fuelMetadataPath -Force }
    }
    if (-not $SkipBuild) {
        & pnpm tauri build --ci @fuelSigningArgs
        if ($LASTEXITCODE -ne 0) { throw 'Release build failed.' }
    } else {
        # Rebundle so skipped compilation cannot reuse an installer with old signing settings.
        & pnpm tauri bundle --ci --bundles nsis @fuelSigningArgs
        if ($LASTEXITCODE -ne 0) { throw 'Regular installer packaging failed.' }
    }
    $fuelInstaller = Join-Path $fuelRoot "target/release/bundle/nsis/TokenFuel_${fuelVersion}_x64-setup.exe"
    if (-not (Test-Path -LiteralPath $fuelInstaller)) { throw 'Regular installer is missing.' }
    Copy-Item -LiteralPath $fuelInstaller -Destination $fuelOut -Force
    & pnpm tauri bundle --ci --config src-tauri/tauri.offline.conf.json --bundles nsis @fuelSigningArgs
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
    Copy-Item -LiteralPath Test-TokenFuel.ps1 -Destination $fuelStage -Force
    $fuelSignatures = foreach ($fuelSignedFile in @(
        (Join-Path $fuelOut "TokenFuel_${fuelVersion}_x64-setup.exe"),
        (Join-Path $fuelOut "TokenFuel_${fuelVersion}_x64-offline-setup.exe"),
        (Join-Path $fuelStage 'TokenFuel.exe')
    )) {
        $fuelSignature = Get-AuthenticodeSignature -LiteralPath $fuelSignedFile
        if (-not $AllowUnsigned -and ($fuelSignature.Status -ne 'Valid' -or -not $fuelSignature.TimeStamperCertificate)) {
            throw "A valid timestamped Authenticode signature is required: $fuelSignedFile (status: $($fuelSignature.Status)). No release checksums generated."
        }
        [ordered]@{
            File = [IO.Path]::GetFileName($fuelSignedFile)
            Status = [string]$fuelSignature.Status
            Subject = $(if ($fuelSignature.SignerCertificate) { $fuelSignature.SignerCertificate.Subject } else { $null })
            Thumbprint = $(if ($fuelSignature.SignerCertificate) { $fuelSignature.SignerCertificate.Thumbprint } else { $null })
            Timestamped = [bool]$fuelSignature.TimeStamperCertificate
        }
    }
    if (-not $AllowUnsigned -and @($fuelSignatures.Thumbprint | Select-Object -Unique).Count -ne 1) {
        throw 'The installers and portable executable must have the same publisher signing certificate.'
    }
    $fuelSignatures | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath (Join-Path $fuelOut 'SIGNATURES.json') -Encoding UTF8
    if ($AllowUnsigned) { Write-Warning 'Unsigned test packaging: not certified for managed Windows deployment.' }
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
    & (Join-Path $PSScriptRoot 'new-winget-manifest.ps1') -ReleaseDirectory $fuelOut
    Write-Output $fuelOut
} finally { Pop-Location }
