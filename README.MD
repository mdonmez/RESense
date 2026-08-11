# RESense

RESense is a Windows command-line control surface for the Acer Nitro AN515-58. It uses the validated NitroSense/PredatorSense service interface, NitroSense registry state, NitroSense keyboard XML, and Windows session APIs.

The supported surface is intentionally narrow: fans, operation modes, keyboard lighting and keyboard settings, LCD overdrive, and DTS-backed sound presets. Windows and the AN515-58 are the only implemented platform/model combination.

## Safety

RESense validates the detected model before connecting. `--dangerously-allow-any-model` bypasses only that model guard. Pipe availability is the service check: if `PSSvc` is stopped or unavailable, the command fails with concise service guidance and does not report a successful state.

Every supported mutation reads back the resulting state before printing it. Operational failures return a nonzero exit code; successful JSON never hides a failed read as `null`. `null` is reserved for known unsupported state, such as sound when the default output is not on the validated DTS path.

## Usage

```text
resense status [fan|keyboard|mode|display|sound]
resense fan auto|max|custom
resense mode quiet|default|performance
resense keyboard brightness|timeout|static|dynamic|sticky|win-menu
resense display overdrive enable|disable
resense sound <auto|music|movies|voice|strategy|rpg|shooter|custom>
```

See [docs/cli.md](docs/cli.md) for commands and the stable JSON contract. The architecture and ownership boundaries are in [docs/architecture.md](docs/architecture.md). Current status and future work are tracked in [docs/roadmap.md](docs/roadmap.md).

## Supported Audio

The DTS path is validated for internal speakers and wired 3.5 mm Realtek headphones. Bluetooth and other non-DTS paths are intentionally unsupported, matching NitroSense's own surface on this machine.

## Reverse Engineering

The evidence and command catalog are indexed in [docs/re/README.md](docs/re/README.md). Raw transport and unsupported-query probing is available only through the feature-gated developer binary:

```powershell
cargo run --features dev-tools --bin probe -- query service current 13 257
```
