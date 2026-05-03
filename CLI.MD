# CLI Skeleton

```
resense
├── status [--json]
├── fan
│   ├── mode {auto,max}
│   └── speed [--cpu N] [--gpu N] [--cpu-auto] [--gpu-auto]
├── keyboard
│   ├── brightness {1-5}
│   ├── static --zone1..4 HEXCOLORS [--brightness {1-5}]
│   ├── dynamic {breathing,neon,shifting,wave,zoom} --speed {1-5} [--brightness] [--color] [--direction {left,right}]
│   ├── sticky {enable,disable}
│   └── win-menu {enable,disable}
├── mode {quiet,default,performance} [--skip-whispermode]
├── display
│   ├── overdrive {enable,disable}
│   └── backlight-timeout {enable,disable} [--brightness-percent N]
└── sound [--backend {auto,dts,waves}] preset
```

# CLI Design

## `resense status`

### `resense status [--json]`

Read all state.

- `--json` - Print JSON instead of human-readable text, great for scripting and integration

Returns fan (temperatures, RPM, mode), keyboard (lighting, sticky keys, win-menu), mode (operation mode), display (overdrive, backlight timeout), and sound (preset, backend) status.

---

## `resense fan`

### `resense fan mode {auto,max}`

Set global fan mode.

- `auto` - Automatic fan control
- `max` - Maximum fan speed

Updates fan control according to the selected mode.

### `resense fan speed [--cpu N] [--gpu N] [--cpu-auto] [--gpu-auto]`

Set custom fan percentages or per-fan auto. At least one of `--cpu`, `--gpu`, `--cpu-auto`, `--gpu-auto` must be provided. Multiple flags cannot be provided for the same fan.

- `--cpu N` - CPU fan speed percentage (0-100)
- `--gpu N` - GPU fan speed percentage (0-100)
- `--cpu-auto` - Set CPU fan to automatic mode
- `--gpu-auto` - Set GPU fan to automatic mode

Updates fan control according to the selected custom speed or auto setting.

---

## `resense keyboard`

### `resense keyboard brightness {1-5}`

Set keyboard brightness level.

- `level` - Brightness level from 1 (disabled) to 5 (brightest)

Updates keyboard brightness according to the selected level.

### `resense keyboard static --zone1..4 HEXCOLORS`

Set 4-zone static lighting. Each zone accepts a 6-digit hex color (e.g., `FF0000` for red) or `off`. At least one zone must be provided.

- `--zone1 HEXCOLOR` - Zone 1 color
- `--zone2 HEXCOLOR` - Zone 2 color
- `--zone3 HEXCOLOR` - Zone 3 color
- `--zone4 HEXCOLOR` - Zone 4 color

Updates static keyboard lighting according to the selected zone colors.

### `resense keyboard dynamic {breathing,neon,shifting,wave,zoom} --speed {1-5} [--color HEXCOLOR] [--direction {left,right}]`

Set dynamic keyboard lighting effect.

- `mode` - Dynamic effect: `breathing`, `neon`, `shifting`, `wave`, `zoom`
- `--speed {1-5}` - Effect speed (required, 1=slowest, 5=fastest)

Updates dynamic keyboard lighting according to the selected effect, speed, color, and direction. `--color` is required for only `breathing`, `wave`, `shifting`, and `zoom`, but not used by `neon`. `--direction` is required for `wave` and `shifting`, but ignored by other modes. Providing wrong flags for a mode will raise an error.

### `resense keyboard sticky {enable,disable}`

Set or read Sticky Keys.

- `enable` - Enable Sticky Keys
- `disable` - Disable Sticky Keys

Updates Sticky Keys according to the selected setting.

### `resense keyboard win-menu {enable,disable}`

Set or read Windows/Menu key lock.

- `enable` - Lock Windows and Menu keys
- `disable` - Unlock Windows and Menu keys

Updates the Windows/Menu key lock according to the selected setting.

---

### `resense mode {quiet,default,performance}`

Set operation mode.

- `quiet` - Quiet mode
- `default` - Balanced default mode
- `performance` - Highest performance mode

Updates the operation mode according to the selected setting.

---

### `resense display overdrive {enable,disable}`

Set or read LCD Overdrive.

- `enable` - Enable LCD Overdrive
- `disable` - Disable LCD Overdrive

Updates LCD Overdrive according to the selected setting.

---

### `resense display backlight-timeout {enable,disable}`

Set or read keyboard backlight timeout.

- `enable` - Enable backlight timeout
- `disable` - Disable backlight timeout

Updates keyboard backlight timeout according to the selected setting.

---

### `resense sound [--backend {auto,dts,waves}] preset`

Set or read sound preset.

- `preset` - DTS presets: `music`, `movies`, `voice`, `strategy`, `rpg`, `shooter`, `custom`, `auto`

Updates the sound preset according to the selected preset and backend. Not available if headphones are connected.
