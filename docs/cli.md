# CLI

RESense uses direct commands for mutations and one read command for state.

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
│   │   [--speed <1-9>] [--color <hex>] [--direction <from-left|from-right>]
│   ├── sticky {enable|disable}
│   └── win-menu {enable|disable}
├── display
│   └── overdrive {enable|disable}
└── sound
    └── <auto|music|movies|voice|strategy|rpg|shooter|custom>
```

## State

`status` without a target reads the complete supported state. A target reads only that subsystem. `--watch` emits synchronous newline-delimited output using the same schema on every iteration; with `--json`, each iteration is one compact JSON line.

Human output is semantic and direct. JSON is a small public API containing
current state values only.

Full JSON has this shape:

```json
{
  "fan": {
    "mode": "custom",
    "cpu": {
      "temperature_c": 60,
      "rpm": 2400,
      "control": { "mode": "manual", "percent": 70 }
    },
    "gpu": {
      "temperature_c": 45,
      "rpm": 2300,
      "control": { "mode": "auto" }
    }
  },
  "keyboard": {
    "brightness": 5,
    "lighting": {
      "mode": "static",
      "zones": [
        { "enabled": true, "color": "#FF0000" },
        { "enabled": false, "color": "#00FF00" },
        { "enabled": true, "color": "#0000FF" },
        { "enabled": false, "color": "#FFFFFF" }
      ]
    },
    "backlight_timeout": true,
    "sticky_keys": false,
    "win_menu_lock": true
  },
  "mode": "performance",
  "display": true,
  "sound": "music"
}
```

Targeted JSON is the exact corresponding value, without a wrapper:

```powershell
resense status mode --json       # "performance"
resense status display --json    # true, false, or null when unsupported
resense status fan --json        # the fan object
```

Operational or verification failures terminate with a nonzero exit code. `null`
means that a value does not apply to the current hardware configuration.

## Mutations

```powershell
resense fan custom --cpu 70 --gpu-auto
resense mode performance
resense keyboard brightness 5
resense keyboard timeout enable
resense keyboard static --zone1 FF0000 --zone2 off
resense keyboard dynamic wave --speed 5 --color 00FFFF --direction from-left
resense keyboard sticky enable
resense keyboard win-menu disable
resense display overdrive enable
resense sound movies
```

Successful mutations print only the verified resulting state. Quiet mode forces both fans to global auto and blocks subsequent fan control until the mode changes. `keyboard sticky` targets the current Windows session. DTS sound and WhisperMode use a reachable admin-agent session because their controlled state is shared on the validated machine.
