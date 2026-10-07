# Generate reviewable manifests from the final installer bytes. Does not publish or install.
[CmdletBinding()]
param([Parameter(Mandatory = $true)][string]$ReleaseDirectory)
$ErrorActionPreference = 'Stop'
$fuelRoot = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$fuelConfig = Get-Content -LiteralPath (Join-Path $fuelRoot 'src-tauri/tauri.conf.json') -Raw | ConvertFrom-Json
$fuelVersion = $fuelConfig.version
if ($fuelVersion -notmatch '^\d+\.\d+\.\d+$') { throw 'WinGet packaging currently requires a stable three-part release version.' }
$fuelAsset = "TokenFuel_${fuelVersion}_x64-setup.exe"
$fuelInstaller = Join-Path $ReleaseDirectory $fuelAsset
$fuelHasher = [Security.Cryptography.SHA256]::Create()
$fuelStream = [IO.File]::OpenRead([IO.Path]::GetFullPath($fuelInstaller))
try { $fuelHash = ([BitConverter]::ToString($fuelHasher.ComputeHash($fuelStream))).Replace('-', '') }
finally { $fuelStream.Dispose(); $fuelHasher.Dispose() }
$fuelOut = Join-Path $ReleaseDirectory "winget/manifests/t/tejas20/TokenFuel/$fuelVersion"
New-Item -ItemType Directory -Force -Path $fuelOut | Out-Null
@"
# yaml-language-server: `$schema=https://aka.ms/winget-manifest.version.1.6.0.schema.json
PackageIdentifier: tejas20.TokenFuel
PackageVersion: $fuelVersion
DefaultLocale: en-US
ManifestType: version
ManifestVersion: 1.6.0
"@ | Set-Content -LiteralPath (Join-Path $fuelOut 'tejas20.TokenFuel.yaml') -Encoding UTF8
@"
# yaml-language-server: `$schema=https://aka.ms/winget-manifest.defaultLocale.1.6.0.schema.json
PackageIdentifier: tejas20.TokenFuel
PackageVersion: $fuelVersion
PackageLocale: en-US
Publisher: tejas20
PublisherUrl: https://github.com/tejas20
PublisherSupportUrl: https://github.com/tejas20/TokenFuel/issues
PackageName: TokenFuel
PackageUrl: https://github.com/tejas20/TokenFuel
License: MIT
LicenseUrl: https://github.com/tejas20/TokenFuel/blob/v$fuelVersion/LICENSE
ShortDescription: Remaining AI subscription usage, at a glance.
ManifestType: defaultLocale
ManifestVersion: 1.6.0
"@ | Set-Content -LiteralPath (Join-Path $fuelOut 'tejas20.TokenFuel.locale.en-US.yaml') -Encoding UTF8
@"
# yaml-language-server: `$schema=https://aka.ms/winget-manifest.installer.1.6.0.schema.json
PackageIdentifier: tejas20.TokenFuel
PackageVersion: $fuelVersion
InstallerType: nullsoft
Scope: user
InstallModes:
- interactive
- silent
UpgradeBehavior: install
Installers:
- Architecture: x64
  InstallerUrl: https://github.com/tejas20/TokenFuel/releases/download/v$fuelVersion/$fuelAsset
  InstallerSha256: $fuelHash
ManifestType: installer
ManifestVersion: 1.6.0
"@ | Set-Content -LiteralPath (Join-Path $fuelOut 'tejas20.TokenFuel.installer.yaml') -Encoding UTF8
Write-Host "WinGet manifests generated for review: $fuelOut"
Write-Host 'Validate with winget validate --manifest <directory>. Verify the published installer hash before submission to microsoft/winget-pkgs. TokenFuel is not available through WinGet until that submission is accepted.'
