#requires -Version 5.1

$ErrorActionPreference = "Stop"

$InstallPath = Join-Path $env:LOCALAPPDATA "Programs\RESense"
$running = @(Get-Process -Name "resense" -ErrorAction SilentlyContinue)
if ($running.Count -gt 0) {
    throw "Close any running resense.exe process before uninstalling RESense."
}

if (Test-Path -LiteralPath $InstallPath) {
    Remove-Item -LiteralPath $InstallPath -Recurse -Force
}

$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
$userEntries = @()
if (-not [string]::IsNullOrWhiteSpace($userPath)) {
    $userEntries = @($userPath -split ";" | Where-Object { -not [string]::IsNullOrWhiteSpace($_) })
}

$normalizedInstallPath = [System.IO.Path]::GetFullPath($InstallPath).TrimEnd("\")
$remainingEntries = @($userEntries | Where-Object {
    -not [string]::Equals($_.Trim().TrimEnd("\"), $normalizedInstallPath, [StringComparison]::OrdinalIgnoreCase)
})

if ($remainingEntries.Count -ne $userEntries.Count) {
    [Environment]::SetEnvironmentVariable("Path", ($remainingEntries -join ";"), "User")
}

$currentEntries = @($env:Path -split ";" | Where-Object { -not [string]::IsNullOrWhiteSpace($_) })
$currentRemainingEntries = @($currentEntries | Where-Object {
    -not [string]::Equals($_.Trim().TrimEnd("\"), $normalizedInstallPath, [StringComparison]::OrdinalIgnoreCase)
})
$env:Path = ($currentRemainingEntries -join ";")

Write-Output "RESense was uninstalled from $InstallPath."
