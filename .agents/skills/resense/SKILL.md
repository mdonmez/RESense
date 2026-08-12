---
name: resense
description: >
  Control Acer Nitro AN515-58 PredatorSense hardware via the RESense CLI:
  fan control, keyboard lighting, operation mode, display overdrive, sound
  presets, and full system status. Use this skill whenever the user mentions
  RESense, NitroSense, PredatorSense, Acer Nitro hardware, fan control,
  keyboard lighting, or laptop operation modes.
---

# RESense

RESense controls Acer Nitro AN515-58 hardware through the validated
PredatorSense/NitroSense service interface.

## Prerequisites

- `resense.exe` must be on `PATH` or in the current directory.
- The default model check requires an Acer Nitro AN515-58.
- The Acer service and required Windows session context must be available.
- Use `--dangerously-allow-any-model` only when intentionally bypassing the
  model check.

## Public CLI

Use `status` for reads. It accepts no target for the full state or one target
(`fan`, `keyboard`, `mode`, `display`, or `sound`) for a focused read.

```powershell
resense status
resense status fan
resense status --json
resense status fan --json
resense status --watch --interval 2
```

Successful text and JSON output report state only.

### Fans

```powershell
resense fan auto
resense fan max
resense fan custom --cpu 70 --gpu-auto
```

`custom` accepts `--cpu <0-100>`, `--gpu <0-100>`, `--cpu-auto`, and
`--gpu-auto`. At least one option is required, and a fixed percentage cannot
be combined with the corresponding automatic flag.

### Operation mode

```powershell
resense mode quiet
resense mode default
resense mode performance
resense mode performance --skip-whispermode
```

### Keyboard

```powershell
resense keyboard brightness 5
resense keyboard timeout disable
resense keyboard static --zone1 FF0000 --zone2 off
resense keyboard dynamic wave --speed 5 --color 00FFFF --direction from-left
resense keyboard sticky enable
resense keyboard win-menu disable
```

### Display

```powershell
resense display overdrive enable
resense display overdrive disable
```

Keyboard backlight timeout is under `keyboard timeout`, not `display`.

### Sound

```powershell
resense sound auto
resense sound music
resense sound movies
resense sound voice
resense sound strategy
resense sound rpg
resense sound shooter
resense sound custom
```

The public sound command uses the DTS path automatically. Supported outputs
are the internal speakers and wired 3.5 mm Realtek output.

## Developer Probes

Reverse-engineering probes are separate Cargo binaries and are intentionally
not part of the normal `resense --help` command tree.
