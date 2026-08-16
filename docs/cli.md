# CLI

RESense uses direct commands for mutations and one read command for state.

```text
resense
├── --version
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
├── sound
│   └── <auto|music|movies|voice|strategy|rpg|shooter|custom>
└── update
```

## State

`status` without a target reads the complete supported state. A target reads only that subsystem. `--watch` emits synchronous newline-delimited output using the same schema on every iteration; with `--json`, each iteration is one compact JSON line.

Human output is semantic and direct. JSON is a small public API containing current state values only.

Full JSON has this shape:

```json
{
  "fan": {
    "mode": "custom",
    "cpu": {
      "temperature_c": 60,
      "rpm": 2400
    },
    "gpu": {
      "temperature_c": 45,
      "rpm": 2300
    },
    "custom": {
      "cpu": { "mode": "manual", "percent": 70 },
      "gpu": { "mode": "auto", "percent": 50 }
    }
  },
  "keyboard": {
    "brightness": 5,
    "lighting": {
      "mode": "static",
      "static": {
        "zones": [
          { "enabled": true, "color": "#FF0000" },
          { "enabled": false, "color": "#00FF00" },
          { "enabled": true, "color": "#0000FF" },
          { "enabled": false, "color": "#FFFFFF" }
        ]
      },
      "dynamic": {
        "effect": "wave",
        "color": "#00FFFF",
        "speed": 3,
        "direction": "from_left"
      }
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

`fan.mode` is the selector for the control currently applied. The live CPU and GPU telemetry stays under `fan.cpu` and `fan.gpu`, while the fixed `fan.custom.cpu` and `fan.custom.gpu` blocks contain the saved custom settings. The custom percentage is always the saved percentage, even when that custom fan is configured as automatic. Keyboard lighting follows the same pattern: `keyboard.lighting.mode` selects the applied block, while `keyboard.lighting.static` and `keyboard.lighting.dynamic` are always present and keep stable paths. Operational or verification failures terminate with a nonzero exit code. `null` means that a value does not apply to the current hardware configuration.

## Version and Updates

`resense --version` prints the installed version immediately, then checks the latest published stable GitHub release. The check is fresh on every invocation and has a five-second timeout. A network failure, rate limit, unavailable PowerShell, or invalid release response leaves the command successful and prints `Update check unavailable`.

```text
RESense 0.1.0
Up to date
```

When a newer stable release exists, the second line is `Update available: <version>`.

`resense update` performs the same check. It exits successfully without making changes when the installed version is current. When an update exists, it starts a visible PowerShell installer obtained through `irm git.new/resense`, passes the exact running executable path and parent process ID, and reports the final result from that installer. The command updates only the exact running `resense.exe`, so portable, development, and custom executable locations are supported without changing neighboring files or PATH entries.

The update is noninteractive. If `$HOME/.agents/skills/resense/SKILL.md` already exists, it is refreshed from the same release. If it does not exist, `resense update` does not install it. Use `& ([scriptblock]::Create((irm git.new/resense))) -YesSkill` for an unattended skill installation or repair.

An update check failure is fatal for `resense update`, and the existing executable is retained if verification or replacement fails.

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
