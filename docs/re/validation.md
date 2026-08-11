# RESense Expected Results Matrix

This matrix describes the current command contract on the validated AN515-58 surface. Service and registry paths are implementation evidence; successful CLI output reports only the verified resulting state.

## Fan

| Command | Service writes | Registry synchronization | Verified state |
| --- | --- | --- | --- |
| `resense fan auto` | `cmd 15` global-auto payload | `CurrentFanMode=0`; remembered custom percentages preserved | `fan.mode=auto` plus live CPU/GPU RPM and temperature |
| `resense fan max` | `cmd 15` max payload | `CurrentFanMode=1` | `fan.mode=max` plus live CPU/GPU RPM and temperature |
| `resense fan custom --cpu-auto --gpu-auto` | `cmd 15` custom-all-auto payload | `CurrentFanMode=2`; both auto flags set | `fan.mode=custom`; both controls `auto` |
| `resense fan custom --cpu 70 --gpu-auto` | `cmd 15` behavior plus `cmd 16` CPU speed | `CurrentFanMode=2`; CPU 70; GPU auto | `fan.mode=custom`; CPU manual; GPU auto |
| `resense fan custom --cpu-auto --gpu 70` | `cmd 15` behavior plus `cmd 16` GPU speed | `CurrentFanMode=2`; CPU auto; GPU 70 | `fan.mode=custom`; CPU auto; GPU manual |
| `resense fan custom --cpu 70 --gpu 70` | `cmd 15` behavior plus `cmd 16` CPU/GPU speed | `CurrentFanMode=2`; both manual | `fan.mode=custom`; both controls manual |

## Keyboard

| Command | Service writes | Persisted state | Verified state |
| --- | --- | --- | --- |
| `resense keyboard brightness <1..5>` | `cmd 27` | installed NitroSense `Main.xml` brightness fields | brightness from `LightingEffects.brightness` |
| `resense keyboard static ...` | `cmd 27`, `cmd 29`, enabled-zone `cmd 28` | four zone nodes and static mode in `Main.xml` | static mode, four zones, and preserved brightness |
| `resense keyboard dynamic <mode> ...` | `cmd 27` | selected pattern, speed, direction, color, and mode in `Main.xml` | dynamic effect and parameters from XML |
| `resense keyboard timeout enable|disable` | `cmd 17`; getter `cmd 20` | none | timeout state from the live getter |
| `resense keyboard sticky enable|disable` | current-session admin `cmd 2` | HKLM compatibility mirror | current-session Windows Sticky Keys state |
| `resense keyboard win-menu enable|disable` | service `cmd 9`, selector 2; getter `cmd 10/query 0` | HKLM compatibility mirror | live Windows/Menu lock state |

## Mode, Display, Sound

| Command | Service/admin writes | Verification |
| --- | --- | --- |
| `resense mode quiet|default|performance` | service `cmd 30`; admin WhisperMode unless skipped; mode mirror | service `cmd 34/query 11`; quiet also forces fans to global auto |
| `resense display overdrive enable|disable` | service `cmd 9`, selector `0x10` | service `cmd 10/query 0`; unsupported capability is explicit `null` |
| `resense sound <preset>` | shared/admin `cmd 14` | shared/admin `cmd 13`; non-DTS is explicit `null`/error for writes |

The wired 3.5 mm Realtek path is part of the validated DTS surface. Bluetooth and other non-DTS paths are unsupported.

## Re-Validation Procedure

Use this checklist whenever a supported feature changes.

1. Record the starting state with `cargo run --bin resense -- status --json`.
2. Snapshot relevant registry keys under `HKLM\SOFTWARE\OEM\NitroSense`
   and files under `C:\ProgramData\OEM\NitroSense`.
3. Change exactly one setting in NitroSense.
4. Re-read RESense state.
5. Apply the equivalent RESense command.
6. Compare NitroSense UI, physical behavior, service/admin readback, and
   registry/XML changes.

For each command, record the CLI operation, payload encoding, persistence
changes, readback source, NitroSense-visible result, physical result, and a
representative raw reply when a getter is involved.

### Minimum Matrix

- Fans: live RPM/temperature, exact active mode, global auto, max, mixed
  custom auto/manual states, and remembered percentages.
- Keyboard: XML-backed static/dynamic mode, brightness, zones, effect,
  direction, speed, color, and timeout.
- Display: overdrive enable and disable with NitroSense confirmation.
- Modes: default, performance, quiet, mode readback, and WhisperMode side
  effect in a hybrid/NVIDIA-App environment.
- Sound: auto, music, and shooter on internal speakers and, when available,
  wired 3.5 mm Realtek output.

### Failure Conditions

If `PSSvc` / `Predator Service` is unavailable, or no matching admin-agent
pipe is reachable, the feature is unavailable and the command must fail. Any
unsupported area must be reported plainly rather than inferred.
