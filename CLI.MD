# CLI Reference

The public CLI uses direct commands for changes and one read-only command for
state:

```text
resense
├── status [fan|keyboard|mode|display|sound]
│   [--json] [--watch] [--interval <seconds>]
├── fan
│   ├── auto
│   ├── max
│   └── custom [--cpu <percent>|--cpu-auto] [--gpu <percent>|--gpu-auto]
├── mode {quiet|default|performance} [--skip-whispermode]
├── keyboard
│   ├── brightness <1-5>
│   ├── timeout {enable|disable}
│   ├── static [--zone1 <hex|off>] [--zone2 <hex|off>] [--zone3 <hex|off>] [--zone4 <hex|off>]
│   ├── dynamic <breathing|neon|shifting|wave|zoom>
│   │   [--speed <1-9>] [--color <hex>] [--direction <fromleft|fromright>]
│   ├── sticky {enable|disable}
│   └── win-menu {enable|disable}
├── display
│   └── overdrive {enable|disable}
└── sound
    └── <auto|music|movies|voice|strategy|rpg|shooter|custom>
```

Developer reverse-engineering probes are separate Cargo binaries and are not
part of the normal `resense --help` command tree.

## Reading State

`status` without a target reads all supported state. Supplying one target
reads only that subsystem:

```text
resense status
resense status fan
resense status keyboard
resense status mode
resense status display
resense status sound
```

Use `--json` for a stable state object. Successful text and JSON output report
what state exists, not how RESense obtained it. They do not include source
paths, pipe names, query numbers, registry keys, reliability labels, or other
provenance metadata.

```text
resense status --json
resense status fan --json
```

`--watch` repeats the selected read. `--interval` sets the polling interval
in seconds and requires `--watch`; JSON watch output emits one compact object
per line.

```text
resense status --watch --interval 2
resense status fan --json --watch --interval 1
```

Example full JSON shape:

```json
{
  "fan": {
    "cpu_temperature_c": 60,
    "gpu_temperature_c": 45,
    "cpu_fan_rpm": 2400,
    "gpu_fan_rpm": 2300,
    "mode": "custom",
    "custom": {
      "cpu": { "percent": 70, "auto": false },
      "gpu": { "percent": 50, "auto": true }
    }
  },
  "keyboard": {
    "lighting": {},
    "backlight_timeout": true,
    "sticky": false,
    "win_menu": true
  },
  "mode": "performance",
  "display": { "overdrive": true },
  "sound": { "preset": "music" }
}
```

Unavailable subsystem state is represented as `null` or as `null` component
values. Errors from a write or an invalid command are reported separately on
stderr.

## Fans

```text
resense fan auto
resense fan max
resense fan custom --cpu 70 --gpu-auto
resense fan custom --cpu-auto --gpu 80
```

`custom` requires at least one CPU/GPU option. A fixed percentage and the
corresponding `--*-auto` flag cannot be used together. Fan control remains
blocked while operation mode is `quiet`, matching the existing safety policy.

## Mode

```text
resense mode quiet
resense mode default
resense mode performance
resense mode performance --skip-whispermode
```

Operation mode writes retain the existing WhisperMode integration unless
`--skip-whispermode` is supplied.

## Keyboard

```text
resense keyboard brightness 5
resense keyboard timeout disable
resense keyboard static --zone1 FF0000 --zone2 off --zone3 00FF00 --zone4 off
resense keyboard dynamic wave --speed 5 --color 00FFFF --direction fromleft
resense keyboard sticky enable
resense keyboard win-menu disable
```

Static lighting preserves the current brightness and unspecified zones.
Dynamic lighting preserves current effect values when possible and falls back
to validated defaults. `--color` is unsupported for `neon`; `--direction` is
only used by `wave` and `shifting`.

## Display

```text
resense display overdrive enable
resense display overdrive disable
```

Keyboard backlight timeout is intentionally under `keyboard timeout`; it is
not a display command.

## Sound

```text
resense sound auto
resense sound music
resense sound movies
resense sound voice
resense sound strategy
resense sound rpg
resense sound shooter
resense sound custom
```

The public command uses the validated DTS path automatically. The supported
outputs are the internal speakers and the validated 3.5 mm Realtek output.
Bluetooth and other non-DTS paths remain unsupported.

## Migration

This is a clean command move. The removed forms are not aliases:

```text
resense get fan                         # removed
resense fan mode auto                   # use resense fan auto
resense fan speed --cpu 70              # use resense fan custom --cpu 70
resense keyboard backlight-timeout ...  # use resense keyboard timeout ...
resense sound --backend dts music       # use resense sound music
```
