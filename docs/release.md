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

The GitHub Actions release job validates the tag, reuses the verified release
binary, creates the ZIP and checksums, renders the version-pinned installer,
and publishes the GitHub Release.

The first release uses `v0.1.0`. Later releases use the same process with the
new version. Existing tags and releases must never be moved or overwritten.

## User URLs

Latest installation:

```powershell
irm https://github.com/mdonmez/RESense/releases/latest/download/install.ps1 | iex
```

Pinned installation:

```powershell
irm https://github.com/mdonmez/RESense/releases/download/v0.1.0/install.ps1 | iex
```

Latest uninstall:

```powershell
irm https://github.com/mdonmez/RESense/releases/latest/download/uninstall.ps1 | iex
```

The installer and uninstaller are user-scoped. They do not install services,
modify NitroSense, or require administrator privileges.
