# Integration State Matrix

This matrix records how the device layer obtains and verifies supported state.

| Feature | State read | State write | Verification |
| --- | --- | --- | --- |
| Fan RPM and temperature | Live vendor telemetry | None | Compared with NitroSense telemetry |
| Fan mode, live telemetry, and custom percentages | NitroSense fan settings plus telemetry | Vendor fan commands and persisted settings | Readback of the global selector, live readings, and custom settings |
| Keyboard lighting | NitroSense `Main.xml` | Vendor lighting commands and atomic XML update | Parse the resulting XML and read state |
| Keyboard timeout | Live vendor getter | Live vendor setter | Live readback |
| Sticky Keys | Current-session Windows state | Current-session vendor operation and persisted mirror | Current-session Windows readback |
| Windows/Menu lock | Live vendor getter | Vendor setter and persisted mirror | Live readback |
| Operation mode | Live vendor mode state | Vendor mode setter, optional WhisperMode, and persisted mirror | Live mode readback |
| LCD overdrive | Capability and live vendor state | Vendor setter | Live readback |
| DTS sound preset | Shared admin-backed state | Shared admin-backed setter | Shared readback |

## State Boundaries

- Keyboard lighting state is read from the installed NitroSense keyboard profile because that is the validated state store for this feature.
- Sticky Keys follows the current Windows session. DTS sound and WhisperMode follow the shared behavior observed on the validated machine.
- Sound validation covers built-in speakers and wired 3.5 mm Realtek output.

## Readback Rules

- Operation-mode writes are verified with the live mode query.
- Fan state keeps the global selector, live telemetry, and saved custom settings in fixed namespaces. Global `auto` or `max` determines the applied behavior without changing the custom paths.
- Keyboard zone state is read from the NitroSense profile.
