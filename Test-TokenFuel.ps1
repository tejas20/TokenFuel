# Read-only Windows launch diagnostics. No downloads, application launch, or provider data access.
[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)][string]$ExecutablePath,
    [string]$ReportPath,
    [Nullable[int]]$NativeErrorCode
)
$ErrorActionPreference = 'Stop'
$fuelPath = [IO.Path]::GetFullPath($ExecutablePath)
if (-not $ReportPath) {
    $ReportPath = Join-Path ([IO.Path]::GetTempPath()) ('TokenFuel-diagnostics-' + [Guid]::NewGuid().ToString('N') + '.json')
}
$fuelReport = [ordered]@{
    ReportVersion = 1
    CreatedUtc = [DateTime]::UtcNow.ToString('o')
    WindowsVersion = [Environment]::OSVersion.Version.ToString()
    Is64BitOperatingSystem = [Environment]::Is64BitOperatingSystem
    PowerShellVersion = $PSVersionTable.PSVersion.ToString()
    LanguageMode = [string]$ExecutionContext.SessionState.LanguageMode
    ExecutablePath = $fuelPath
    NativeErrorCode = $NativeErrorCode
    FileExists = Test-Path -LiteralPath $fuelPath -PathType Leaf
    Sha256 = $null
    SignatureStatus = 'NotChecked'
    SignerSubject = $null
    SignerThumbprint = $null
    InternetZoneId = $null
    WebView2Versions = @()
    PolicyEvents = @()
    Checks = @()
}
if ($fuelReport.FileExists) {
    try {
        $fuelHasher = [Security.Cryptography.SHA256]::Create()
        $fuelStream = [IO.File]::OpenRead($fuelPath)
        try { $fuelReport.Sha256 = ([BitConverter]::ToString($fuelHasher.ComputeHash($fuelStream))).Replace('-', '') }
        finally { $fuelStream.Dispose(); $fuelHasher.Dispose() }
    } catch { $fuelReport.Checks += 'Executable could not be read for hashing.' }
    try {
        $fuelSignature = Get-AuthenticodeSignature -LiteralPath $fuelPath
        $fuelReport.SignatureStatus = [string]$fuelSignature.Status
        if ($fuelSignature.SignerCertificate) {
            $fuelReport.SignerSubject = $fuelSignature.SignerCertificate.Subject
            $fuelReport.SignerThumbprint = $fuelSignature.SignerCertificate.Thumbprint
        }
    } catch { $fuelReport.Checks += 'Authenticode signature could not be read.' }
    # Record only the zone number, never the download/referrer URLs.
    $fuelZone = Get-Content -LiteralPath $fuelPath -Stream Zone.Identifier -ErrorAction SilentlyContinue
    foreach ($fuelLine in $fuelZone) {
        if ($fuelLine -match '^ZoneId=(\d+)$') { $fuelReport.InternetZoneId = [int]$Matches[1] }
    }
}
foreach ($fuelRegistry in @(
    'HKCU:\Software\Microsoft\EdgeUpdate\Clients\*',
    'HKLM:\Software\Microsoft\EdgeUpdate\Clients\*',
    'HKLM:\Software\WOW6432Node\Microsoft\EdgeUpdate\Clients\*'
)) {
    try {
        $fuelClients = @(Get-ItemProperty -Path $fuelRegistry -ErrorAction Stop)
        foreach ($fuelClient in $fuelClients) {
            if ($fuelClient.name -like '*WebView2*' -and $fuelClient.pv -and $fuelClient.pv -ne '0.0.0.0') {
                $fuelReport.WebView2Versions += [string]$fuelClient.pv
            }
        }
    } catch { $fuelReport.Checks += "WebView2 registry location absent or unreadable: $fuelRegistry" }
}
$fuelReport.WebView2Versions = @($fuelReport.WebView2Versions | Select-Object -Unique)
foreach ($fuelLog in @('Microsoft-Windows-AppLocker/EXE and DLL', 'Microsoft-Windows-CodeIntegrity/Operational')) {
    try {
        # Bounded, read-only query. Do not include raw event messages or unrelated apps.
        $fuelEvents = Get-WinEvent -FilterHashtable @{ LogName = $fuelLog; StartTime = (Get-Date).AddHours(-1); Id = @(8003,8004,3033,3034,3076,3077) } -MaxEvents 100 -ErrorAction Stop
        foreach ($fuelEvent in $fuelEvents) {
            $fuelXml = [xml]$fuelEvent.ToXml()
            $fuelMatches = @($fuelXml.Event.EventData.Data | Where-Object {
                $_.InnerText -and ($_.InnerText -eq $fuelPath -or $_.InnerText -match ('(?i)[\\/]' + [Regex]::Escape([IO.Path]::GetFileName($fuelPath)) + '$'))
            })
            if ($fuelMatches.Count -gt 0) {
                $fuelReport.PolicyEvents += [ordered]@{ Log = $fuelLog; Id = $fuelEvent.Id; TimeUtc = $fuelEvent.TimeCreated.ToUniversalTime().ToString('o') }
            }
        }
    } catch { $fuelReport.Checks += "No accessible recent events in $fuelLog (no matches, log unavailable, or access denied)." }
}
$fuelReport.Checks += 'No matching events does not prove that execution is allowed. IT may need endpoint-security logs. Events matched by filename can refer to another copy of the executable.'
if ($fuelReport.WebView2Versions.Count -eq 0) { $fuelReport.Checks += 'WebView2 was not detected in standard registry locations. This is separate from a Windows access-denied launch error.' }
$fuelReport | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $ReportPath -Encoding UTF8
Write-Host "Diagnostic report: $ReportPath"
Write-Host "Executable: $fuelPath"
Write-Host "Signature: $($fuelReport.SignatureStatus); SHA-256: $($fuelReport.Sha256)"
Write-Host 'The report contains local paths and Windows metadata. Review it before sharing with IT. It contains no provider credentials or usage data.'
Write-Output ([IO.Path]::GetFullPath($ReportPath))
