# Releases

RESense releases are Windows x64 artifacts built from explicit Git tags.

## Release A Maintainer

1. Update the package version in `Cargo.toml` and keep `Cargo.lock` consistent.
2. Run the local verification commands:

   ```powershell
   cargo fmt --check
   cargo clippy --all-targets --all-features -- -D warnings
   cargo test --all-targets --all-features
   cargo build --release --locked
   ```

3. Merge the change to `main` after CI succeeds.
4. Create and push an annotated tag matching the package version:

   ```powershell
   git tag -a v0.1.0 -m "RESense v0.1.0"
   git push origin v0.1.0
   ```

The GitHub Actions release job validates the tag, reuses the verified release binary, creates a ZIP containing only `resense.exe`, publishes the version-matched `SKILL.md`, creates checksums for every asset, renders the version-pinned installer, and publishes the GitHub Release.

The first release uses `v0.1.0`. Later releases use the same process with the new version. Existing tags and releases must never be moved or overwritten.

## User URLs

Latest installation:

```powershell
irm git.new/resense | iex
```

The interactive installer asks whether to install the optional agent skill. To install the binary and skill without prompting, run:

```powershell
& ([scriptblock]::Create((irm git.new/resense))) -YesSkill
```

or

```powershell
irm https://github.com/mdonmez/RESense/releases/latest/download/install.ps1 | iex
```

git.new/resense URL will redirect to the explicit latest release, it is created using [Dub](https://dub.co/) for making installation command compacter.

---

Pinned installation:

```powershell
irm https://github.com/mdonmez/RESense/releases/download/v0.1.0/install.ps1 | iex
```

---

Latest uninstall:

```powershell
irm https://github.com/mdonmez/RESense/releases/latest/download/uninstall.ps1 | iex
```

The installer and uninstaller are user-scoped. The installer can install the matching skill release asset at `$HOME/.agents/skills/resense/SKILL.md`. They do not install services, modify NitroSense, or require administrator privileges.

## Self-Update

`resense --version` reports the installed version and performs a fresh check for the latest published stable release. `resense update` uses the same source and does nothing when the installed version is current.

When a newer release exists, the command starts a visible installer process using `irm git.new/resense`. The installer receives the exact running executable path and parent process ID, waits for RESense to exit, verifies the release ZIP, and replaces only that executable. It does not change PATH, install directories, services, registry settings, or neighboring files. This also supports portable, development, and custom executable locations.

The update is noninteractive. If the skill already exists, the installer refreshes it from the same release. If it does not exist, the update leaves it absent.

The existing executable is preserved when the release cannot be verified or the replacement is blocked. The updater reports the final download, verification, replacement, or failure result independently after the command hands it off.

Future release ZIPs contain only `resense.exe`; `SHA256SUMS.txt`, `install.ps1`, `uninstall.ps1`, and the version-matched `SKILL.md` remain separate release assets. The skill URL for a release is `https://github.com/mdonmez/RESense/releases/download/v{VERSION}/SKILL.md`.
