# RESense

RESense is a Windows command-line control surface for the Acer Nitro AN515-58.

It manages fans, operation modes, keyboard lighting and settings, LCD overdrive,
and DTS sound presets. Windows and the AN515-58 are the supported platform and
model.

## Safety

RESense validates the detected model before connecting.
`--dangerously-allow-any-model` bypasses only that model check. The required
Acer service must be available.

Supported mutations verify the resulting state before printing it. Operational
failures return a nonzero exit code. A JSON `null` means that a value does not
apply to the current hardware configuration.

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

## Installation

Latest release installation and removal instructions are in
[docs/release.md](docs/release.md). The release workflow publishes Windows x64
artifacts from explicit `vX.Y.Z` tags.

## Audio

Sound presets are supported through the DTS path for the built-in speakers and
wired 3.5 mm Realtek headphones.

## Project Notes

Contributor research is indexed in [docs/re/README.md](docs/re/README.md).
