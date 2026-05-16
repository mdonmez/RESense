# RESense Reverse Engineering Notes

## Current Context

NitroSense stopped launching after the 2026-04 Windows update cycle on this AN515-58, which forced RESense to become the practical control surface for fan, keyboard, mode, display, and sound features.

As of 2026-05-15, after KB5089549, NitroSense is accessible again on this machine. RESense development continues, but the restored GUI now gives a stronger live reverse-engineering path: NitroSense can be used as the reference UI while RESense observes and validates the same behavior.

## Captures

- `captures/2026-05-16-nitrosense-gui.md` - first non-invasive capture from the restored NitroSense GUI using process inspection, UI Automation, registry/XML snapshots, named-pipe enumeration, and `resense status`.

## Live Comparison Workflow

Use one NitroSense UI action per experiment.

1. Record the starting RESense state with `resense status --json`.
2. Snapshot relevant NitroSense-owned state:
   - `HKLM\SOFTWARE\OEM\NitroSense`
   - `C:\ProgramData\OEM\NitroSense`
3. Change exactly one setting in NitroSense.
4. Capture the affected service/admin named-pipe command if possible.
5. Re-read RESense status and the same registry/XML paths.
6. Apply the equivalent RESense command.
7. Compare NitroSense UI state, RESense status, registry/XML state, and physical hardware behavior.

## State Sources

- Service pipe: `\\.\pipe\PredatorSense_service_namedpipe`
- Admin-agent pipe prefix: `\\.\pipe\PredatorSense_admin_agent_`
- Service process: `PSSvc` / `Predator Service`
- NitroSense registry root: `HKLM\SOFTWARE\OEM\NitroSense`
- Lighting profile XML root: `C:\ProgramData\OEM\NitroSense\ProfilePool\LightProfilePool`

## Current Assumptions To Re-Verify

- Fan RPM and CPU/GPU temperatures are trusted live service values.
- Fan mode has a known live ambiguity: true global Auto and some mixed custom states can both read as `auto_like`.
- Keyboard brightness, static/dynamic mode, and RGB colors are currently treated as NitroSense persisted XML state because no trusted live getter has been found.
- Keyboard zone enabled/disabled state has a useful live getter.
- Operation mode, LCD Overdrive, Windows/Menu lock, and parts of backlight timeout have live readback paths.

## Experiment Rules

- Keep each test narrow and repeatable.
- Prefer live hardware proof for fan behavior, especially RPM changes.
- Treat NitroSense XML and registry values as persisted UI state unless the test proves they match live hardware.
- Record return codes and raw payloads when a command succeeds with a non-obvious reply.
- Do not copy NitroSense UI restrictions into RESense unless they are required for hardware safety or correctness.
