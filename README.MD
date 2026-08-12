# RESense

RESense is an open-source command-line alternative to NitroSense for Acer Nitro AN515-58 laptops.

It provides a direct way to control and check fans, power modes, keyboard lighting, display overdrive, and DTS sound presets on Windows. These features have been tested on the AN515-58 and may not work on other laptops.

## Capabilities

- **Fan control** with automatic, maximum, and per-fan manual modes, plus live readings.
- **Power modes** include quiet, default, and performance.
- **Keyboard controls** include brightness, backlight timeout, four-zone static lighting, dynamic effects, Sticky Keys, and Windows/Menu lock.
- **LCD overdrive**.
- **DTS sound presets** work with internal speakers and wired 3.5 mm Realtek output.
- **Status** shows everything or one part of the laptop, in regular text or JSON, and can keep updating on screen.

## Installation

Open Terminal (cmd/PowerShell) and run to install RESense:

```powershell
irm git.new/resense | iex
```

<details>
<summary>Install a specific version</summary>

Use the release-specific installer link if you need a particular version:

```powershell
irm github.com/mdonmez/RESense/releases/download/v0.1.0/install.ps1 | iex
```

</details>

<details>
<summary>Uninstall RESense</summary>

Remove installation with:

```powershell
irm github.com/mdonmez/RESense/releases/latest/download/uninstall.ps1 | iex
```

</details>

## Agent Skill

RESense includes an agent skill that teaches AI coding tools how to control your laptop.

Install it with:

```powershell
npx skills add mdonmez/RESense
```

Just describe what you want. Your agent can help installing and updating RESense, checking the current state, and controlling your computer.

<details>
<summary>Skill Usage Examples</summary>

> Make my laptop quieter. → `resense mode quiet`

> I'm going to do heavy work. → `resense mode performance`

> Reduce my keyboard brightness. → `resense keyboard brightness 2`

> Cool down my laptop. → `resense fan custom --cpu 80 --gpu 80`

> Is my laptop overheating? → `resense status --json`

</details>

## Quick Start

```powershell
resense status
resense mode performance
resense fan custom --cpu 70 --gpu-auto
resense keyboard brightness 5
resense sound music
resense --help
```

See the [CLI reference](docs/cli.md) for the full list of commands and the
JSON format.

## Safety and Support

RESense can change hardware settings. Use it at your own risk. Incorrect use, unsupported hardware, service or driver changes, power loss, or interrupted operations may cause overheating, instability, data loss, hardware damage, or loss of functionality.

RESense is developed and validated on the Acer Nitro 5 AN515-58-79CQ. It is intended and tested only for Acer Nitro 5 AN515-58 series with the Acer service available. It is provided without any guarantee or warranty of compatibility, safety, accuracy, availability, or fitness for a particular purpose.

The default model check helps prevent accidental use on unsupported laptops. `--dangerously-allow-any-model` deliberately bypasses that protection and should be treated as unsafe. Do not use it unless you understand and accept the risks.

You are solely responsible for every command, setting, and result. Monitor your laptop, stop using RESense if anything behaves unexpectedly, and keep your own backups. The authors and contributors accept no responsibility or liability for damage, data loss, hardware problems, or other consequences resulting from the use of this software. Support for other laptops or configurations is not guaranteed.

## Development

```powershell
git clone https://github.com/mdonmez/RESense.git
cd RESense
cargo test --all-targets --all-features
cargo build --release --locked
```

The hardware test must be run manually on a supported AN515-58.

For how the project works and how to contribute, see:

- [Architecture](docs/architecture.md)
- [Hardware tests](docs/hardware-tests.md)
- [Reverse-engineering notes](docs/re/README.md)
- [Roadmap](docs/roadmap.md)
- [Release process](docs/release.md)
