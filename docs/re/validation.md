# Feature Validation Matrix

This matrix defines the expected verified result for each supported command on the Acer Nitro AN515-58.

## Fans

| Command | Device operation | Verified result |
| --- | --- | --- |
| `resense fan auto` | Global automatic control | `fan.mode=auto`, with live CPU/GPU readings and unchanged `fan.custom.*` settings |
| `resense fan max` | Maximum control | `fan.mode=max`, with live CPU/GPU readings and unchanged `fan.custom.*` settings |
| `resense fan custom --cpu-auto --gpu-auto` | Custom mode, both automatic | `fan.mode=custom`, both `fan.custom.*.mode` values are `auto` |
| `resense fan custom --cpu 70 --gpu-auto` | CPU manual, GPU automatic | `fan.custom.cpu.mode=manual`, `fan.custom.cpu.percent=70`, and GPU mode `auto` |
| `resense fan custom --cpu-auto --gpu 70` | CPU automatic, GPU manual | CPU mode `auto`, `fan.custom.gpu.mode=manual`, and GPU percentage `70` |
| `resense fan custom --cpu 70 --gpu 70` | Both manual | Both `fan.custom.*.mode` values are `manual` with percentage `70` |

Automatic control preserves each fan's saved custom percentage.

The saved custom settings remain visible when global `auto` or `max` is active. Use `fan.mode` to determine the behavior currently applied.

## Keyboard

| Command | Device operation | Verified result |
| --- | --- | --- |
| `resense keyboard brightness <1..5>` | Set keyboard brightness | Brightness is updated in the keyboard profile and on the device |
| `resense keyboard static ...` | Set selected static zones | Four-zone static state and unchanged fields are preserved |
| `resense keyboard dynamic <mode> ...` | Set effect parameters | Effect, speed, direction, color, and brightness are read back |
| `resense keyboard timeout enable|disable` | Set keyboard timeout | Timeout state is read back |
| `resense keyboard sticky enable|disable` | Set Sticky Keys | Current Windows session is read back |
| `resense keyboard win-menu enable|disable` | Set Windows/Menu lock | Live lock state is read back |

## Mode, Display, And Sound

| Command | Device operation | Verified result |
| --- | --- | --- |
| `resense mode quiet|default|performance` | Set operation mode | Mode is read back; quiet mode enables automatic fan control |
| `resense display overdrive enable|disable` | Set LCD overdrive | Capability and resulting state are read back |
| `resense sound <preset>` | Set DTS preset | Preset is read back on supported DTS output |

Sound validation covers built-in speakers and wired 3.5 mm Realtek output.

## Revalidation Procedure

1. Record `resense status --json`.
2. Snapshot NitroSense registry values and keyboard profile files.
3. Change one setting in NitroSense.
4. Read RESense state.
5. Apply the equivalent RESense command.
6. Compare application state, persisted state, and physical behavior.

Record protocol details in [protocol.md](protocol.md) only when a change affects the device integration.

## Minimum Hardware Matrix

- Fans: telemetry, auto, max, mixed custom control, and saved custom percentages.
- Keyboard: static and dynamic lighting, brightness, zones, effects, direction, speed, color, and timeout.
- Display: overdrive enable and disable.
- Modes: quiet, default, performance, and WhisperMode behavior when available.
- Sound: auto, music, and shooter on built-in speakers and wired output.

If the Acer service or required admin context is unavailable, the command must return an error. Hardware capabilities that do not apply to the current device are reported as unavailable.
