use crate::error::Result;
use anyhow::{Context, bail};
use serde::Deserialize;
use std::ffi::OsString;
use std::fmt;
use std::io::Write;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

const CHECK_TIMEOUT: Duration = Duration::from_secs(5);

const LATEST_RELEASE_SCRIPT: &str = r#"
$ErrorActionPreference = 'Stop'
$headers = @{
    Accept = 'application/vnd.github+json'
    'User-Agent' = 'RESense-version-check'
}
$release = Invoke-RestMethod -UseBasicParsing -Method Get -Uri 'https://api.github.com/repos/mdonmez/RESense/releases/latest' -Headers $headers -TimeoutSec 5
[pscustomobject]@{
    tag_name = [string]$release.tag_name
    draft = [bool]$release.draft
    prerelease = [bool]$release.prerelease
} | ConvertTo-Json -Compress
"#;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Version {
    major: u64,
    minor: u64,
    patch: u64,
}

impl Version {
    fn current() -> Self {
        Self::parse_tag(concat!("v", env!("CARGO_PKG_VERSION")))
            .expect("Cargo package version must be a stable vX.Y.Z version")
    }

    fn parse_tag(tag: &str) -> Result<Self> {
        let version = tag
            .strip_prefix('v')
            .with_context(|| format!("release tag '{tag}' does not start with 'v'"))?;
        let components: Vec<_> = version.split('.').collect();
        if components.len() != 3 {
            bail!("release tag '{tag}' is not a strict vX.Y.Z version")
        }

        let mut parsed = [0_u64; 3];
        for (index, component) in components.iter().enumerate() {
            if component.is_empty()
                || !component.bytes().all(|byte| byte.is_ascii_digit())
                || (component.len() > 1 && component.starts_with('0'))
            {
                bail!("release tag '{tag}' is not a strict vX.Y.Z version")
            }
            parsed[index] = component
                .parse()
                .with_context(|| format!("release tag '{tag}' contains an invalid number"))?;
        }

        Ok(Self {
            major: parsed[0],
            minor: parsed[1],
            patch: parsed[2],
        })
    }
}

impl fmt::Display for Version {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

#[derive(Debug, Deserialize)]
struct ReleasePayload {
    tag_name: String,
    draft: bool,
    prerelease: bool,
}

#[derive(Debug)]
struct LatestRelease {
    tag: String,
    version: Version,
}

pub fn version_requested(args: &[OsString]) -> bool {
    let mut arguments = args.iter().skip(1);
    arguments
        .clone()
        .any(|argument| argument == "--version" || argument == "-V")
        && arguments.all(|argument| {
            argument == "--version"
                || argument == "-V"
                || argument == "--dangerously-allow-any-model"
        })
}

pub fn print_version() -> Result<()> {
    let current = Version::current();
    println!("RESense {current}");
    std::io::stdout()
        .flush()
        .context("could not flush version output")?;

    match latest_release() {
        Ok(latest) if latest.version > current => {
            println!("Update available: {}", latest.version);
        }
        Ok(_) => println!("Up to date"),
        Err(_) => println!("Update check unavailable"),
    }

    Ok(())
}

pub fn run_update() -> Result<()> {
    let current = Version::current();
    let latest = latest_release().context("update check unavailable")?;

    println!("RESense {current}");
    if latest.version <= current {
        println!("Up to date");
        return Ok(());
    }

    println!("Update available: {}", latest.version);
    handoff_update(&latest.tag)?;
    println!("Update handed off");
    Ok(())
}

fn latest_release() -> Result<LatestRelease> {
    parse_release(&run_powershell_capture(LATEST_RELEASE_SCRIPT)?)
}

fn parse_release(payload: &str) -> Result<LatestRelease> {
    let release: ReleasePayload =
        serde_json::from_str(payload.trim()).context("GitHub returned malformed release data")?;
    if release.draft || release.prerelease {
        bail!("GitHub latest release is not a published stable release")
    }

    let version = Version::parse_tag(&release.tag_name)?;
    Ok(LatestRelease {
        tag: release.tag_name,
        version,
    })
}

fn run_powershell_capture(script: &str) -> Result<String> {
    let mut child = Command::new("powershell.exe")
        .args([
            "-NoLogo",
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            script,
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("PowerShell is unavailable")?;
    let deadline = Instant::now() + CHECK_TIMEOUT;

    loop {
        if child
            .try_wait()
            .context("could not poll PowerShell")?
            .is_some()
        {
            let output = child
                .wait_with_output()
                .context("could not read PowerShell output")?;
            if !output.status.success() {
                let details = String::from_utf8_lossy(&output.stderr).trim().to_owned();
                if details.is_empty() {
                    bail!("PowerShell exited with {}", output.status)
                }
                bail!("PowerShell exited with {}: {details}", output.status)
            }
            return String::from_utf8(output.stdout)
                .context("PowerShell returned non-UTF-8 output");
        }

        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            bail!("version check timed out after five seconds")
        }

        thread::sleep(Duration::from_millis(25));
    }
}

fn handoff_update(tag: &str) -> Result<()> {
    let target = std::env::current_exe().context("could not resolve the running executable")?;
    let script = updater_script(tag, &target.to_string_lossy(), std::process::id());

    Command::new("powershell.exe")
        .args([
            "-NoLogo",
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            &script,
        ])
        .spawn()
        .context("could not start the updater; PowerShell is unavailable")?;

    Ok(())
}

fn updater_script(tag: &str, target: &str, parent_process_id: u32) -> String {
    let script = r#"
$ErrorActionPreference = 'Stop'
$repository = 'mdonmez/RESense'
$releaseTag = __RELEASE_TAG__
$targetExecutable = __TARGET_EXECUTABLE__
$parentProcessId = __PARENT_PROCESS_ID__
$temporaryRoot = Join-Path ([System.IO.Path]::GetTempPath()) ('resense-update-' + [guid]::NewGuid().ToString('N'))
$baseUrl = "https://github.com/$repository/releases/download/$releaseTag"
$apiUrl = "https://api.github.com/repos/$repository/releases/latest"

function Get-Sha256Hex {
    param(
        [Parameter(Mandatory = $true)]
        [string]$FilePath
    )

    $algorithm = $null
    $stream = $null
    try {
        $algorithm = [System.Security.Cryptography.SHA256]::Create()
        $stream = [System.IO.File]::OpenRead($FilePath)
        $hashBytes = $algorithm.ComputeHash($stream)
        return ([System.BitConverter]::ToString($hashBytes)).Replace('-', '').ToUpperInvariant()
    }
    finally {
        if ($null -ne $stream) {
            $stream.Dispose()
        }
        if ($null -ne $algorithm) {
            $algorithm.Dispose()
        }
    }
}

try {
    New-Item -ItemType Directory -Force -Path $temporaryRoot | Out-Null
    $headers = @{
        Accept = 'application/vnd.github+json'
        'User-Agent' = 'RESense-updater'
    }
    $release = Invoke-RestMethod -UseBasicParsing -Method Get -Uri $apiUrl -Headers $headers -TimeoutSec 5
    if ([string]$release.tag_name -ne $releaseTag -or [bool]$release.draft -or [bool]$release.prerelease) {
        throw "The latest GitHub release changed before the update started."
    }

    $checksumsPath = Join-Path $temporaryRoot 'SHA256SUMS.txt'
    $installerPath = Join-Path $temporaryRoot 'install.ps1'
    Invoke-WebRequest -UseBasicParsing -TimeoutSec 30 -Uri "$baseUrl/SHA256SUMS.txt" -OutFile $checksumsPath
    Invoke-WebRequest -UseBasicParsing -TimeoutSec 30 -Uri "$baseUrl/install.ps1" -OutFile $installerPath

    $checksumMatches = @(Get-Content -LiteralPath $checksumsPath | Where-Object {
        $parts = $_.Trim() -split '\s+', 2
        $parts.Count -eq 2 -and $parts[1].Trim() -eq 'install.ps1'
    })
    if ($checksumMatches.Count -ne 1) {
        throw "SHA-256 entry for install.ps1 was not found or was ambiguous."
    }

    $expectedHash = (($checksumMatches[0].Trim() -split '\s+', 2)[0]).ToUpperInvariant()
    $actualHash = Get-Sha256Hex -FilePath $installerPath
    if ($actualHash -ne $expectedHash) {
        throw 'SHA-256 verification failed for install.ps1.'
    }

    & $installerPath -TargetExecutable $targetExecutable -ParentProcessId $parentProcessId
    if ($LASTEXITCODE -and $LASTEXITCODE -ne 0) {
        exit $LASTEXITCODE
    }
}
catch {
    Write-Error ('RESense update failed: ' + $_.Exception.Message)
    exit 1
}
finally {
    if (Test-Path -LiteralPath $temporaryRoot) {
        Remove-Item -LiteralPath $temporaryRoot -Recurse -Force -ErrorAction SilentlyContinue
    }
}
"#;

    script
        .replace("__RELEASE_TAG__", &powershell_literal(tag))
        .replace("__TARGET_EXECUTABLE__", &powershell_literal(target))
        .replace("__PARENT_PROCESS_ID__", &parent_process_id.to_string())
}

fn powershell_literal(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_stable_versions() {
        assert_eq!(
            Version::parse_tag("v0.1.0").unwrap(),
            Version {
                major: 0,
                minor: 1,
                patch: 0
            }
        );
        assert_eq!(Version::parse_tag("v1.2.3").unwrap().to_string(), "1.2.3");
        assert!(Version::parse_tag("v42.7.19").is_ok());
    }

    #[test]
    fn rejects_non_stable_version_tags() {
        for tag in [
            "1.2.3",
            "v1.2",
            "v1.2.3.4",
            "v1.2.3-alpha",
            "v1.2.3+build",
            "v01.2.3",
            "v1.two.3",
            "v1.2.-3",
        ] {
            assert!(Version::parse_tag(tag).is_err(), "accepted {tag}");
        }
    }

    #[test]
    fn compares_versions_numerically() {
        assert!(Version::parse_tag("v1.2.4").unwrap() > Version::parse_tag("v1.2.3").unwrap());
        assert_eq!(
            Version::parse_tag("v1.2.3").unwrap(),
            Version::parse_tag("v1.2.3").unwrap()
        );
        assert!(Version::parse_tag("v1.2.2").unwrap() < Version::parse_tag("v1.2.3").unwrap());
    }

    #[test]
    fn accepts_only_published_stable_release_metadata() {
        let release =
            parse_release(r#"{"tag_name":"v1.2.3","draft":false,"prerelease":false}"#).unwrap();
        assert_eq!(release.version.to_string(), "1.2.3");

        assert!(
            parse_release(r#"{"tag_name":"v1.2.4","draft":false,"prerelease":true}"#,).is_err()
        );
        assert!(
            parse_release(r#"{"tag_name":"v1.2.5","draft":true,"prerelease":false}"#,).is_err()
        );
        assert!(parse_release("not-json").is_err());
    }

    #[test]
    fn recognizes_version_arguments_before_clap() {
        assert!(version_requested(&[
            OsString::from("resense"),
            OsString::from("--version"),
        ]));
        assert!(version_requested(&[
            OsString::from("resense"),
            OsString::from("-V"),
        ]));
        assert!(!version_requested(&[
            OsString::from("resense"),
            OsString::from("update"),
        ]));
        assert!(!version_requested(&[
            OsString::from("resense"),
            OsString::from("status"),
            OsString::from("--version"),
        ]));
        assert!(!version_requested(&[
            OsString::from("resense"),
            OsString::from("--version"),
            OsString::from("unexpected"),
        ]));
    }

    #[test]
    fn updater_targets_only_the_running_executable() {
        let script = updater_script("v1.2.3", r"C:\Tools\resense.exe", 42);
        assert!(script.contains("-TargetExecutable"));
        assert!(script.contains(r"'C:\Tools\resense.exe'"));
        assert!(script.contains("$parentProcessId = 42"));
        assert!(script.contains("install.ps1"));
        assert!(script.contains("SHA-256 verification failed for install.ps1"));
        assert!(script.contains("function Get-Sha256Hex"));
        assert!(!script.contains("Get-FileHash"));
    }
}
