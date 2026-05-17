# RESense Validation Checklist

Use this checklist whenever a supported feature changes.

## One-Setting-at-a-Time Procedure

1. Record the starting RESense state:
   - `cargo run --bin resense -- status --json`
2. Snapshot relevant persisted state:
   - registry keys under `HKLM\SOFTWARE\OEM\NitroSense`
   - `C:\ProgramData\OEM\NitroSense` XML/profile files
3. Change exactly one setting in NitroSense.
4. Re-read RESense state.
5. Apply the equivalent RESense command.
6. Compare:
   - NitroSense UI
   - physical hardware behavior
   - service/admin readback
   - registry/XML changes

## What To Record Per Command

- CLI command
- service/admin command code and payload encoding
- registry writes, if any
- XML writes, if any
- readback source
- NitroSense-visible result
- physical hardware result, if applicable
- representative raw replies when a getter is involved

## Supported Surface Re-Validation

### Fan

- verify live RPM/temp still read from `cmd 13`
- verify exact active mode still matches HKLM `FanControl`
- verify at least:
  - `fan mode auto`
  - `fan mode max`
  - `fan speed --cpu 70 --gpu-auto`
  - `fan speed --cpu-auto --gpu 70`
  - `fan speed --cpu-auto --gpu-auto`

### Keyboard

- verify XML-backed keyboard state still matches NitroSense + hardware for:
  - static vs dynamic mode
  - brightness
  - static zones
  - dynamic effect/direction/speed/color
- verify at least:
  - `keyboard brightness 5`
  - `keyboard static --zone1 0000FF`
  - `keyboard dynamic breathing --speed 9 --color 00FFFF`
  - `keyboard backlight-timeout enable`

### Display

- verify:
  - `display overdrive disable`
  - `display overdrive enable`
- confirm NitroSense advanced settings matches service readback

### Operation Mode

- verify:
  - `mode default`
  - `mode performance`
  - `mode quiet`
- confirm `cmd 34/query 11` and HKLM `CurrentOperationMode`
- in a hybrid/NVIDIA-App environment, also confirm WhisperMode side effect for `quiet`

### Sound

- on the supported DTS/internal-speaker path, verify:
  - `sound auto`
  - `sound music`
  - `sound shooter`
- confirm admin getter readback after each set

## Known Unsupported / Deferred Areas

- independent live keyboard-state getter
- wired `3.5 mm` Waves sound path
- exact meaning of every successful `cmd 20` / `cmd 34` query outside supported decodes
- full multi-session admin-agent behavior

## Failure-Mode Notes

- if `PSSvc` / `Predator Service` is not running, service-pipe features are unavailable
- if no matching admin-agent session pipe is reachable, admin-backed features fail
- when a feature is outside the supported surface, RESense should report that plainly rather than inferring a fake supported state
