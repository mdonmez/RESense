---
name: resense
description: >
  Control Acer Nitro AN515-52 PredatorSense hardware via a CLI — fan speed
  and mode, keyboard backlight and lighting zones, operation mode (quiet /
  default / performance), display overdrive, and sound presets. Use this
  skill whenever the user mentions resense, resense.exe, NitroSense,
  PredatorSense, Acer Nitro fan control, keyboard lighting, or any Acer
  gaming laptop hardware control.
---

# resense

CLI to read and write Acer Nitro AN515-58 hardware state via the
PredatorSense / NitroSense service interface.

## Prerequisites

- `resense.exe` must be available in the current directory or on `PATH`
- Acer Nitro AN515-58 laptop (use `--dangerously-allow-any-model` to bypass)
- Some commands require admin privileges (fan speed, sound)

## Global Options

| Option | Description |
|--------|-------------|
| `--dangerously-allow-any-model` | Bypass the AN515-58 model check and run on any machine |
| `-h`, `--help` | Print help |

## Commands

### `resense status`

Read all current hardware state (fan, keyboard, mode, display, sound).

```bash
resense status [OPTIONS]
```

**Options:**

| Option | Description |
|--------|-------------|
| `--json` | Print JSON instead of human-readable text |
| `--dangerously-allow-any-model` | Bypass model check |

---

### `resense fan mode`

Set the global fan mode.

```bash
resense fan mode <MODE>
```

**Arguments:**

| Arg | Values | Description |
|-----|--------|-------------|
| `MODE` | `auto`, `max` | Fan mode |

---

### `resense fan speed`

Set CPU and/or GPU fan speeds or switch to automatic mode.

```bash
resense fan speed [OPTIONS]
```

**Options:**

| Option | Description |
|--------|-------------|
| `--cpu <PERCENT>` | CPU fan speed percentage |
| `--gpu <PERCENT>` | GPU fan speed percentage |
| `--cpu-auto` | Set CPU fan to automatic mode |
| `--gpu-auto` | Set GPU fan to automatic mode |

> At least one option is required. Mixing fixed speed with auto on the
> same fan is not allowed.

---

### `resense keyboard brightness`

Set keyboard backlight brightness.

```bash
resense keyboard brightness <LEVEL>
```

**Arguments:**

| Arg | Description |
|-----|-------------|
| `LEVEL` | Brightness level 1–5 |

---

### `resense keyboard backlight-timeout`

Enable or disable the keyboard backlight timeout.

```bash
resense keyboard backlight-timeout <STATE>
```

**Arguments:**

| Arg | Values | Description |
|-----|--------|-------------|
| `STATE` | `enable`, `disable` | Backlight timeout state |

---

### `resense keyboard static`

Set 4-zone static keyboard lighting colors.

```bash
resense keyboard static [OPTIONS]
```

**Options:**

| Option | Description |
|--------|-------------|
| `--zone1 <COLOR>` | Zone 1 color as 6-digit hex or `off` |
| `--zone2 <COLOR>` | Zone 2 color as 6-digit hex or `off` |
| `--zone3 <COLOR>` | Zone 3 color as 6-digit hex or `off` |
| `--zone4 <COLOR>` | Zone 4 color as 6-digit hex or `off` |

Colors are hex without `#` prefix (e.g. `FF69B4`).

---

### `resense keyboard dynamic`

Set a dynamic keyboard lighting effect.

```bash
resense keyboard dynamic <MODE> [OPTIONS]
```

**Arguments:**

| Arg | Values | Description |
|-----|--------|-------------|
| `MODE` | `breathing`, `neon`, `shifting`, `wave`, `zoom` | Effect type |

**Options:**

| Option | Default | Description |
|--------|---------|-------------|
| `--speed <1-9>` | — | Effect speed |
| `--color <HEX>` | — | Effect color as 6-digit hex |
| `--direction` | — | `from-left` or `from-right` |

---

### `resense keyboard sticky`

Enable or disable Sticky Keys.

```bash
resense keyboard sticky <STATE>
```

**Arguments:**

| Arg | Values | Description |
|-----|--------|-------------|
| `STATE` | `enable`, `disable` | Sticky Keys state |

---

### `resense keyboard win-menu`

Enable or disable Windows/Menu key lock.

```bash
resense keyboard win-menu <STATE>
```

**Arguments:**

| Arg | Values | Description |
|-----|--------|-------------|
| `STATE` | `enable`, `disable` | Win/Menu key lock state |

---

### `resense mode`

Set the operation mode.

```bash
resense mode <MODE> [OPTIONS]
```

**Arguments:**

| Arg | Values | Description |
|-----|--------|-------------|
| `MODE` | `quiet`, `default`, `performance` | Operation mode |

**Options:**

| Option | Description |
|--------|-------------|
| `--skip-whispermode` | Skip WhisperMode integration |

---

### `resense display overdrive`

Enable or disable LCD overdrive.

```bash
resense display overdrive <STATE>
```

**Arguments:**

| Arg | Values | Description |
|-----|--------|-------------|
| `STATE` | `enable`, `disable` | LCD overdrive state |

---

### `resense sound`

Set the sound preset.

```bash
resense sound <PRESET> [OPTIONS]
```

**Arguments:**

| Arg | Values | Description |
|-----|--------|-------------|
| `PRESET` | `music`, `movies`, `voice`, `strategy`, `rpg`, `shooter`, `custom`, `auto` | Sound preset |

**Options:**

| Option | Values | Description |
|--------|--------|-------------|
| `--backend` | `auto`, `dts` | Audio backend to use |

## Examples

```bash
# Read full hardware status
resense status

# Read status as JSON (for scripting)
resense status --json

# Set quiet mode for battery / low noise
resense mode quiet

# Switch to performance mode
resense mode performance

# Set both fans to auto (default controlled)
resense fan speed --cpu-auto --gpu-auto

# Force fans to 80%
resense fan speed --cpu 80 --gpu 80

# Set fan mode to max cooling
resense fan mode max

# Set keyboard brightness to 3
resense keyboard brightness 3

# Zone 1 pink, zone 2 pink, zones 3-4 off
resense keyboard static --zone1 FF69B4 --zone2 FF69B4 --zone3 off --zone4 off

# Breathing effect in red
resense keyboard dynamic breathing --speed 5 --color FF0000

# Enable LCD overdrive
resense display overdrive enable

# Set sound preset to movies
resense sound movies
```

## Quick Reference

| Task | Command |
|------|---------|
| Read all state | `resense status` |
| Read all state as JSON | `resense status --json` |
| Set quiet mode | `resense mode quiet` |
| Set performance mode | `resense mode performance` |
| Fans to auto | `resense fan speed --cpu-auto --gpu-auto` |
| Fans to max cooling | `resense fan mode max` |
| Set fan speeds manually | `resense fan speed --cpu 70 --gpu 70` |
| Keyboard brightness | `resense keyboard brightness 4` |
| Static zone colors | `resense keyboard static --zone1 FF0000 --zone2 00FF00 --zone3 0000FF --zone4 FFFFFF` |
| Dynamic effect | `resense keyboard dynamic wave --speed 7 --color FF69B4` |
| Backlight timeout | `resense keyboard backlight-timeout disable` |
| Enable LCD overdrive | `resense display overdrive enable` |
| Set sound preset | `resense sound movies` |
| Run on any laptop | `resense --dangerously-allow-any-model status` |
