# Reverse-Engineering Notes

Contributor reference for the Windows integration with Acer NitroSense and PredatorSense. These notes record observed vendor interfaces, state storage, and validation results; they are not an additional user-facing API.

## Contents

- [protocol.md](protocol.md): service and admin command encodings used by the device layer.
- [state.md](state.md): the state model and persistence matrix for each feature.
- [validation.md](validation.md): repeatable feature validation procedures.
- [captures/2026-05-16-nitrosense-gui.md](captures/2026-05-16-nitrosense-gui.md): a dated NitroSense UI and state capture.
- [linux-port.md](linux-port.md): exploratory Linux hardware-interface notes.

## Investigation Method

Use one NitroSense action per experiment.

1. Record `resense status --json`.
2. Snapshot the NitroSense registry and keyboard profile files.
3. Change one setting in NitroSense.
4. Capture the corresponding vendor command when useful.
5. Read RESense state again.
6. Apply the equivalent RESense command.
7. Compare application state, persisted state, and physical behavior.

Keep experiments reversible and record protocol details in [protocol.md](protocol.md) when they affect the device layer.

## Integration Summary

| Feature | Observed state | Validation basis |
| --- | --- | --- |
| Fans | Live telemetry plus persisted fan control state | NitroSense UI, live readings, and reversible hardware cycles |
| Keyboard lighting | NitroSense keyboard profile XML | UI, XML, service writes, and physical lighting |
| Keyboard timeout | Live vendor getter/setter | NitroSense toggle and readback |
| Sticky Keys | Current Windows session | Windows state and session-specific validation |
| Windows/Menu lock | Live vendor getter/setter | NitroSense state and readback |
| Operation mode | Live mode state with WhisperMode integration | NitroSense modes, NVIDIA App, and readback |
| LCD overdrive | Capability and live state | NitroSense setting and readback |
| DTS sound | Built-in speakers and wired 3.5 mm Realtek output | NitroSense presets, audio playback, and readback |

## Maintenance Rules

- Keep the public CLI contract in [../cli.md](../cli.md), not in research notes.
- Keep protocol constants and payload explanations in [protocol.md](protocol.md), not in user-facing output documentation.
- Do not expose a vendor command, session identifier, or transport detail as product state.
- Add a feature to the supported surface only after it has a typed state model, a write path, readback, and reversible validation.
