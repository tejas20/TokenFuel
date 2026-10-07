# Regression checks under Windows PowerShell 5.1. No downloads or app launches.
$ErrorActionPreference = 'Stop'
$fuelRoot = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
function Assert-Fuel($Condition, [string]$Message) {
    if (-not $Condition) { throw $Message }
}
foreach ($fuelScript in @('Start-TokenFuel.ps1', 'Test-TokenFuel.ps1', 'scripts/package-release.ps1', 'scripts/new-winget-manifest.ps1')) {
    $fuelTokens = $null; $fuelErrors = $null
    [void][Management.Automation.Language.Parser]::ParseFile((Join-Path $fuelRoot $fuelScript), [ref]$fuelTokens, [ref]$fuelErrors)
    Assert-Fuel ($fuelErrors.Count -eq 0) "PowerShell parse errors in $fuelScript`: $fuelErrors"
}
$fuelTempBase = [IO.Path]::GetFullPath([IO.Path]::GetTempPath()).TrimEnd('\', '/') + [IO.Path]::DirectorySeparatorChar
$fuelTestRoot = Join-Path $fuelTempBase ('TokenFuel-tests-' + [Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $fuelTestRoot | Out-Null
try {
    $fuelLauncher = Join-Path $fuelRoot 'Start-TokenFuel.ps1'
    $fuelSource = Get-Content -LiteralPath $fuelLauncher -Raw
    $fuelVersion = [Regex]::Match($fuelSource, "\`$fuelVersion = '([^']+)'").Groups[1].Value
    $fuelRevision = [Regex]::Match($fuelSource, "\`$fuelRevision = '([^']+)'").Groups[1].Value
    Assert-Fuel ($fuelVersion -and $fuelRevision) 'Cannot determine launcher cache version.'
    $fuelCache = Join-Path $fuelTestRoot "TokenFuel/Preview/$fuelVersion/$fuelRevision"
    New-Item -ItemType Directory -Path $fuelCache -Force | Out-Null
    $fuelExe = Join-Path $fuelCache 'TokenFuel.exe'
    Set-Content -LiteralPath $fuelExe -Value 'test fixture, never executed' -Encoding ascii
    $fuelHash = (Get-FileHash -LiteralPath $fuelExe -Algorithm SHA256).Hash
    Set-Content -LiteralPath (Join-Path $fuelCache 'executable.sha256') -Value $fuelHash
    $fuelDriver = Join-Path $fuelTestRoot 'driver.ps1'
    @'
param($Launcher, $TestRoot, $Mode)
$ErrorActionPreference = 'Stop'
$global:LASTEXITCODE = 0
$env:LOCALAPPDATA = $TestRoot
$env:TEMP = $TestRoot
$env:TMP = $TestRoot
function Invoke-WebRequest { throw 'TEST: download attempted' }
function Get-WinEvent { throw 'TEST: event log unavailable' }
function Get-ItemProperty { throw 'TEST: registry unavailable' }
function Start-Process {
    param($FilePath, $WorkingDirectory, $WindowStyle, $ErrorAction)
    if ($Mode -eq 'denied') { throw (New-Object ComponentModel.Win32Exception 5) }
    if (-not (Test-Path -LiteralPath $FilePath)) { throw 'Executable missing' }
    if ($WorkingDirectory -ne [IO.Path]::GetDirectoryName($FilePath)) { throw 'Wrong working directory' }
    Set-Content -LiteralPath (Join-Path $TestRoot 'launched.txt') -Value $FilePath
}
switch ($Mode) {
    'diagnose' { & $Launcher -Diagnose }
    'downloadOnly' { & $Launcher -DownloadOnly }
    default { & $Launcher }
}
exit $LASTEXITCODE
'@ | Set-Content -LiteralPath $fuelDriver -Encoding UTF8
    foreach ($fuelMode in @('diagnose', 'downloadOnly', 'cached', 'denied')) {
        $fuelMarker = Join-Path $fuelTestRoot 'launched.txt'
        if (Test-Path -LiteralPath $fuelMarker) { Remove-Item -LiteralPath $fuelMarker }
        # cmd/powershell stderr can produce NativeCommandError records in 5.1; inspect the exit code explicitly.
        $fuelPreviousPreference = $ErrorActionPreference
        $ErrorActionPreference = 'Continue'
        $fuelOutput = & powershell.exe -NoProfile -File $fuelDriver $fuelLauncher $fuelTestRoot $fuelMode 2>&1
        $fuelExit = $LASTEXITCODE
        $ErrorActionPreference = $fuelPreviousPreference
        Assert-Fuel ($fuelExit -eq $(if ($fuelMode -eq 'denied') { 1 } else { 0 })) "Unexpected exit for $fuelMode`: $fuelOutput"
        Assert-Fuel ((Test-Path -LiteralPath $fuelMarker) -eq ($fuelMode -eq 'cached')) "Unexpected launch for $fuelMode"
        if ($fuelMode -eq 'denied') {
            Assert-Fuel (($fuelOutput | Out-String) -match 'Windows error code: 5') 'Native access-denied code not reported.'
        }
    }
    $fuelReports = @(Get-ChildItem -LiteralPath $fuelTestRoot -Filter 'TokenFuel-diagnostics-*.json' | ForEach-Object { Get-Content -LiteralPath $_.FullName -Raw | ConvertFrom-Json })
    Assert-Fuel ($fuelReports.Count -eq 2) 'Diagnose and failed launch should each save a report.'
    Assert-Fuel (@($fuelReports | Where-Object Sha256 -eq $fuelHash).Count -eq 2) 'Diagnostic executable hashes do not match.'
    Assert-Fuel (@($fuelReports | Where-Object NativeErrorCode -eq 5).Count -eq 1) 'Report lost native error code.'
    # Corrupt cache must never reach Start-Process; the mock download is deliberately refused.
    Remove-Item -LiteralPath $fuelMarker -ErrorAction SilentlyContinue
    Add-Content -LiteralPath $fuelExe -Value 'tampered'
    $ErrorActionPreference = 'Continue'
    $fuelOutput = & powershell.exe -NoProfile -File $fuelDriver $fuelLauncher $fuelTestRoot 'cached' 2>&1
    $fuelExit = $LASTEXITCODE
    $ErrorActionPreference = 'Stop'
    Assert-Fuel ($fuelExit -ne 0 -and -not (Test-Path -LiteralPath $fuelMarker)) 'Tampered executable was launched.'
    Assert-Fuel (($fuelOutput | Out-String) -match 'TEST: download attempted') 'Tampered cache did not enter the verified download path.'
    # Diagnose must work even when the cache is absent, with no network access.
    Remove-Item -LiteralPath $fuelExe
    & powershell.exe -NoProfile -File $fuelDriver $fuelLauncher $fuelTestRoot 'diagnose' | Out-Null
    Assert-Fuel ($LASTEXITCODE -eq 0) 'Missing-cache diagnostics failed.'
    # Signed packaging is mandatory unless explicitly opting into an unsigned test build.
    $fuelRejected = $false
    try { & (Join-Path $fuelRoot 'scripts/package-release.ps1') -SigningConfig '' }
    catch { $fuelRejected = $_.Exception.Message -like '*Release packaging requires*' }
    Assert-Fuel $fuelRejected 'Packaging silently accepted missing signing configuration.'
    $fuelConfig = Get-Content -LiteralPath (Join-Path $fuelRoot 'src-tauri/tauri.conf.json') -Raw | ConvertFrom-Json
    $fuelFixture = Join-Path $fuelTestRoot "TokenFuel_$($fuelConfig.version)_x64-setup.exe"
    Set-Content -LiteralPath $fuelFixture -Value 'final installer fixture' -Encoding ascii
    & (Join-Path $fuelRoot 'scripts/new-winget-manifest.ps1') -ReleaseDirectory $fuelTestRoot
    $fuelManifests = Join-Path $fuelTestRoot "winget/manifests/t/tejas20/TokenFuel/$($fuelConfig.version)"
    Assert-Fuel (@(Get-ChildItem -LiteralPath $fuelManifests -Filter '*.yaml').Count -eq 3) 'Expected three WinGet manifests.'
    $fuelInstallerManifest = Get-Content -LiteralPath (Join-Path $fuelManifests 'tejas20.TokenFuel.installer.yaml') -Raw
    Assert-Fuel ($fuelInstallerManifest.Contains((Get-FileHash -LiteralPath $fuelFixture).Hash)) 'WinGet hash is not calculated from final installer bytes.'
    Assert-Fuel ($fuelInstallerManifest -match 'Scope: user' -and $fuelInstallerManifest -match 'InstallerType: nullsoft') 'Wrong WinGet installer semantics.'
    # Exercise release verification with mocked signatures/bundlers in an isolated fixture repo.
    # This proves the gate and config forwarding, not real certificate signing.
    $fuelFixtureRoot = Join-Path $fuelTestRoot 'signing-fixture'
    foreach ($fuelRelativeFile in @('scripts/package-release.ps1', 'scripts/new-winget-manifest.ps1', 'package.json', 'src-tauri/tauri.conf.json', 'Test-TokenFuel.ps1', 'LICENSE', 'THIRD_PARTY_NOTICES.md', 'public/providers/LICENSE-octicons', 'public/providers/LICENSE-lobe-icons', 'docs/design/LICENSE-phosphor', 'docs/portable-readme.txt')) {
        $fuelDestination = Join-Path $fuelFixtureRoot $fuelRelativeFile
        New-Item -ItemType Directory -Path ([IO.Path]::GetDirectoryName($fuelDestination)) -Force | Out-Null
        Copy-Item -LiteralPath (Join-Path $fuelRoot $fuelRelativeFile) -Destination $fuelDestination
    }
    $fuelFixtureSigning = Join-Path $fuelFixtureRoot 'signing.json'
    '{"bundle":{"windows":{"signCommand":{"cmd":"mock-signer","args":["%1"]}}}}' | Set-Content -LiteralPath $fuelFixtureSigning
    foreach ($fuelSigningMode in @('unsigned', 'noTimestamp', 'mixedSigner', 'valid')) {
        $fuelBundleCalls = New-Object 'Collections.Generic.List[string]'
        $fuelFixtureOut = Join-Path $fuelFixtureRoot "artifacts/release/v$($fuelConfig.version)"
        New-Item -ItemType Directory -Path $fuelFixtureOut -Force | Out-Null
        Set-Content -LiteralPath (Join-Path $fuelFixtureOut 'SHA256SUMS.txt') -Value 'stale checksums'
        $fuelSigningFailure = $null
        try {
            & {
                function pnpm {
                    Assert-Fuel ($args -contains $fuelFixtureSigning) 'Signing configuration was not passed to a bundler.'
                    $fuelBundleCalls.Add(($args -join ' '))
                    $fuelBundleFolder = Join-Path $fuelFixtureRoot 'target/release/bundle/nsis'
                    New-Item -ItemType Directory -Path $fuelBundleFolder -Force | Out-Null
                    Set-Content -LiteralPath (Join-Path $fuelBundleFolder "TokenFuel_$($fuelConfig.version)_x64-setup.exe") -Value 'mock installer'
                    Set-Content -LiteralPath (Join-Path $fuelFixtureRoot 'target/release/tokenfuel.exe') -Value 'mock application'
                    $global:LASTEXITCODE = 0
                }
                function Get-AuthenticodeSignature {
                    param($LiteralPath)
                    [pscustomobject]@{
                        Status = $(if ($fuelSigningMode -eq 'unsigned') { 'NotSigned' } else { 'Valid' })
                        TimeStamperCertificate = $(if ($fuelSigningMode -eq 'noTimestamp') { $null } else { 'mock timestamp' })
                        SignerCertificate = [pscustomobject]@{
                            Subject = 'Mock Publisher'
                            Thumbprint = $(if ($fuelSigningMode -eq 'mixedSigner' -and $LiteralPath -like '*offline*') { 'B' } else { 'A' })
                        }
                    }
                }
                & (Join-Path $fuelFixtureRoot 'scripts/package-release.ps1') -SkipBuild -SigningConfig $fuelFixtureSigning | Out-Null
            }
        } catch { $fuelSigningFailure = $_.Exception.Message }
        Assert-Fuel ($fuelBundleCalls.Count -eq 2) "Both bundlers must receive signing configuration ($fuelSigningMode): $fuelSigningFailure"
        if ($fuelSigningMode -eq 'valid') {
            Assert-Fuel (-not $fuelSigningFailure) "Valid mocked signatures rejected: $fuelSigningFailure"
            Assert-Fuel (Test-Path -LiteralPath (Join-Path $fuelFixtureOut 'SHA256SUMS.txt')) 'Signed packaging did not produce checksums.'
        } else {
            Assert-Fuel ($fuelSigningFailure -match 'signature|signing certificate') "Expected signature rejection ($fuelSigningMode): $fuelSigningFailure"
            Assert-Fuel (-not (Test-Path -LiteralPath (Join-Path $fuelFixtureOut 'SHA256SUMS.txt'))) 'Failed signing left stale checksums.'
        }
    }
    Write-Host 'PASS: launcher cache, no-launch diagnostics, access-denied reporting, tamper rejection, signing gates/config forwarding, and WinGet hashes.'
} finally {
    $fuelResolvedTestRoot = [IO.Path]::GetFullPath($fuelTestRoot)
    if ($fuelResolvedTestRoot.StartsWith($fuelTempBase, [StringComparison]::OrdinalIgnoreCase) -and [IO.Path]::GetFileName($fuelResolvedTestRoot).StartsWith('TokenFuel-tests-')) {
        Remove-Item -LiteralPath $fuelResolvedTestRoot -Recurse -Force
    }
}
