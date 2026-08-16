use crate::error::Result;
use anyhow::{Context, bail};
use serde::Deserialize;
use std::ffi::OsString;
use std::fmt;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const CHECK_TIMEOUT: Duration = Duration::from_secs(5);
const UPDATE_TIMEOUT: Duration = Duration::from_secs(300);
const REPOSITORY: &str = "mdonmez/RESense";

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
    version: Version,
    tag: String,
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
    println!("Updating RESense...");

    running_executable()?;
    let existing_skill = existing_skill_path()?;
    let temporary_root = create_update_directory()?;
    let staged_binary = temporary_root.join("resense.exe");
    let staged_skill = existing_skill
        .as_ref()
        .map(|_| temporary_root.join("SKILL.md"));
    let installer_path = temporary_root.join("install.ps1");
    let result = update_files(
        &latest,
        &temporary_root,
        &installer_path,
        &staged_binary,
        staged_skill.as_deref(),
    );

    let cleanup_result = fs::remove_dir_all(&temporary_root);
    let outcome = match result {
        Ok(outcome) => outcome,
        Err(error) => {
            let _ = cleanup_result;
            return Err(error);
        }
    };
    cleanup_result.context("could not clean up the temporary update directory")?;

    match outcome {
        UpdateOutcome::Complete => {
            println!("Updated RESense from {current} to {}", latest.version);
        }
        UpdateOutcome::SkillFailed(error) => {
            println!("Updated RESense from {current} to {}", latest.version);
            println!("Warning: RESense skill update failed");
            return Err(error);
        }
    }
    Ok(())
}

enum UpdateOutcome {
    Complete,
    SkillFailed(anyhow::Error),
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
        version,
        tag: release.tag_name,
    })
}

fn run_powershell_capture(script: &str) -> Result<String> {
    run_powershell_capture_with_timeout(script, CHECK_TIMEOUT, "version check")
}

fn run_powershell_capture_with_timeout(
    script: &str,
    timeout: Duration,
    operation: &str,
) -> Result<String> {
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
    let deadline = Instant::now() + timeout;

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
            bail!("{operation} timed out")
        }

        thread::sleep(Duration::from_millis(25));
    }
}

fn running_executable() -> Result<PathBuf> {
    let target = std::env::current_exe().context("could not resolve the running executable")?;
    if !target.is_file() {
        bail!(
            "the running executable does not exist: {}",
            target.display()
        )
    }
    Ok(target)
}

fn existing_skill_path() -> Result<Option<PathBuf>> {
    let home = std::env::var_os("USERPROFILE").or_else(|| std::env::var_os("HOME"));
    let Some(home) = home else {
        return Ok(None);
    };
    let path = PathBuf::from(home)
        .join(".agents")
        .join("skills")
        .join("resense")
        .join("SKILL.md");
    Ok(path.is_file().then_some(path))
}

fn create_update_directory() -> Result<PathBuf> {
    let base = std::env::temp_dir();
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system clock is before the Unix epoch")?
        .as_nanos();
    for attempt in 0..10 {
        let path = base.join(format!(
            "resense-update-{}-{timestamp}-{attempt}",
            std::process::id()
        ));
        match fs::create_dir(&path) {
            Ok(()) => return Ok(path),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(error).with_context(|| {
                    format!(
                        "could not create temporary update directory {}",
                        path.display()
                    )
                });
            }
        }
    }
    bail!("could not create a unique temporary update directory")
}

fn release_asset_url(tag: &str, asset: &str) -> String {
    format!("https://github.com/{REPOSITORY}/releases/download/{tag}/{asset}")
}

fn update_files(
    release: &LatestRelease,
    temporary_root: &Path,
    installer_path: &Path,
    staged_binary: &Path,
    staged_skill: Option<&Path>,
) -> Result<UpdateOutcome> {
    let script = staging_script(
        release,
        temporary_root,
        installer_path,
        staged_binary,
        staged_skill,
    );
    run_powershell_capture_with_timeout(&script, UPDATE_TIMEOUT, "update staging")?;

    if !staged_binary.is_file() {
        bail!("the verified installer did not produce a staged resense.exe")
    }

    replace_running_executable(staged_binary)
        .context("could not replace the running resense.exe")?;

    if let Some(staged_skill) = staged_skill {
        if !staged_skill.is_file() {
            bail!("the verified installer did not produce a staged SKILL.md")
        }
        let script = skill_commit_script(installer_path, staged_skill);
        if let Err(error) =
            run_powershell_capture_with_timeout(&script, UPDATE_TIMEOUT, "skill update")
        {
            return Ok(UpdateOutcome::SkillFailed(error.context(
                "the binary was updated, but the skill could not be updated",
            )));
        }
    }

    Ok(UpdateOutcome::Complete)
}

#[cfg(windows)]
fn replace_running_executable(staged_binary: &Path) -> Result<()> {
    self_replace::self_replace(staged_binary).context("self-replace failed")?;
    Ok(())
}

#[cfg(not(windows))]
fn replace_running_executable(_staged_binary: &Path) -> Result<()> {
    bail!("self-update is supported only on Windows")
}

fn staging_script(
    release: &LatestRelease,
    temporary_root: &Path,
    installer_path: &Path,
    staged_binary: &Path,
    staged_skill: Option<&Path>,
) -> String {
    let checksums_path = temporary_root.join("SHA256SUMS.txt");
    let installer_url = release_asset_url(&release.tag, "install.ps1");
    let checksums_url = release_asset_url(&release.tag, "SHA256SUMS.txt");
    let stage_skill_argument = staged_skill
        .map(|path| {
            format!(
                "\n    & $installerPath -StageBinaryPath {} -StageSkillPath {}",
                powershell_literal(&staged_binary.to_string_lossy()),
                powershell_literal(&path.to_string_lossy())
            )
        })
        .unwrap_or_else(|| {
            format!(
                "\n    & $installerPath -StageBinaryPath {}",
                powershell_literal(&staged_binary.to_string_lossy())
            )
        });

    r#"
$ErrorActionPreference = 'Stop'
$temporaryRoot = __TEMPORARY_ROOT__
$installerPath = __INSTALLER_PATH__
$checksumsPath = __CHECKSUMS_PATH__
$installerUrl = __INSTALLER_URL__
$checksumsUrl = __CHECKSUMS_URL__

function Get-ExpectedChecksum {
    param(
        [Parameter(Mandatory = $true)][string]$ChecksumsFile,
        [Parameter(Mandatory = $true)][string]$AssetName
    )

    $matches = @(Get-Content -LiteralPath $ChecksumsFile | Where-Object {
        $parts = $_.Trim() -split "\s+", 2
        $parts.Count -eq 2 -and $parts[1].Trim() -eq $AssetName
    })
    if ($matches.Count -ne 1) {
        throw "SHA-256 entry for '$AssetName' was not found or was ambiguous."
    }
    return (($matches[0].Trim() -split "\s+", 2)[0]).ToUpperInvariant()
}

function Get-Sha256Hex {
    param([Parameter(Mandatory = $true)][string]$FilePath)
    $algorithm = $null
    $stream = $null
    try {
        $algorithm = [System.Security.Cryptography.SHA256]::Create()
        $stream = [System.IO.File]::OpenRead($FilePath)
        $hashBytes = $algorithm.ComputeHash($stream)
        return ([System.BitConverter]::ToString($hashBytes)).Replace('-', '').ToUpperInvariant()
    }
    finally {
        if ($null -ne $stream) { $stream.Dispose() }
        if ($null -ne $algorithm) { $algorithm.Dispose() }
    }
}

try {
    New-Item -ItemType Directory -Force -Path $temporaryRoot | Out-Null
    Invoke-WebRequest -UseBasicParsing -TimeoutSec 30 -Uri $installerUrl -OutFile $installerPath
    Invoke-WebRequest -UseBasicParsing -TimeoutSec 30 -Uri $checksumsUrl -OutFile $checksumsPath
    $expectedHash = Get-ExpectedChecksum -ChecksumsFile $checksumsPath -AssetName 'install.ps1'
    $actualHash = Get-Sha256Hex -FilePath $installerPath
    if ($actualHash -ne $expectedHash) {
        throw 'SHA-256 verification failed for install.ps1.'
    }
    __STAGE_ARGUMENTS__
}
catch {
    throw ('RESense update failed: ' + $_.Exception.Message)
}
"#
    .replace(
        "__TEMPORARY_ROOT__",
        &powershell_literal(&temporary_root.to_string_lossy()),
    )
    .replace(
        "__INSTALLER_PATH__",
        &powershell_literal(&installer_path.to_string_lossy()),
    )
    .replace(
        "__CHECKSUMS_PATH__",
        &powershell_literal(&checksums_path.to_string_lossy()),
    )
    .replace("__INSTALLER_URL__", &powershell_literal(&installer_url))
    .replace("__CHECKSUMS_URL__", &powershell_literal(&checksums_url))
    .replace("__STAGE_ARGUMENTS__", stage_skill_argument.trim_start())
}

fn skill_commit_script(installer_path: &Path, staged_skill: &Path) -> String {
    format!(
        r#"
$ErrorActionPreference = 'Stop'
try {{
    & {} -CommitSkillPath {}
}}
catch {{
    throw ('RESense skill update failed: ' + $_.Exception.Message)
}}
"#,
        powershell_literal(&installer_path.to_string_lossy()),
        powershell_literal(&staged_skill.to_string_lossy())
    )
}

fn powershell_literal(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

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
        assert_eq!(release.tag, "v1.2.3");

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
    fn creates_exact_release_asset_urls() {
        assert_eq!(
            release_asset_url("v1.2.3", "install.ps1"),
            "https://github.com/mdonmez/RESense/releases/download/v1.2.3/install.ps1"
        );
        assert_eq!(
            release_asset_url("v1.2.3", "SHA256SUMS.txt"),
            "https://github.com/mdonmez/RESense/releases/download/v1.2.3/SHA256SUMS.txt"
        );
    }

    #[test]
    fn staging_and_commit_scripts_are_valid_powershell() {
        let root = PathBuf::from(r"C:\Temp\resense-update");
        let release = LatestRelease {
            version: Version::parse_tag("v1.2.3").unwrap(),
            tag: "v1.2.3".to_owned(),
        };
        let installer = root.join("install.ps1");
        let binary = root.join("resense.exe");
        let skill = root.join("SKILL.md");
        let script = staging_script(&release, &root, &installer, &binary, Some(&skill));
        assert!(
            script.contains(
                "https://github.com/mdonmez/RESense/releases/download/v1.2.3/install.ps1"
            )
        );
        assert!(script.contains("-StageBinaryPath"));
        assert!(script.contains("-StageSkillPath"));
        assert!(script.contains("Get-Sha256Hex"));
        assert!(!script.contains("git.new"));
        assert!(!script.contains("ParentProcessId"));
        assert!(!script.contains("TargetExecutable"));
        assert!(!script.contains("ScriptBlock::Create"));

        let commit = skill_commit_script(&installer, &skill);
        assert!(commit.contains("-CommitSkillPath"));
        assert!(!commit.contains("git.new"));

        let path =
            std::env::temp_dir().join(format!("resense-update-script-{}.ps1", std::process::id()));
        fs::write(&path, format!("{script}\n{commit}"))
            .expect("could not write PowerShell syntax fixture");

        let command = format!(
            "$source = Get-Content -LiteralPath {} -Raw; [void][scriptblock]::Create($source)",
            powershell_literal(&path.to_string_lossy())
        );
        let output = Command::new("powershell.exe")
            .args([
                "-NoLogo",
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                &command,
            ])
            .output()
            .expect("PowerShell is unavailable");
        let _ = fs::remove_file(&path);

        assert!(
            output.status.success(),
            "invalid generated PowerShell: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
