# RESense Unresolved Assumptions

This file is the explicit holding area for anything not yet fully validated.

Rules:

- If an item is here, it is not settled behavior.
- If an item leaves this file, the corresponding validation should be documented in the main reverse-engineering docs.
- Deferred hardware paths stay here until real hardware is available.

## Open

### Wider Undecoded Query Surface

- Exact meaning of successful `cmd 20` queries outside the supported keyboard backlight-timeout decode
  - status: open but outside the supported surface
- Exact meaning of successful `cmd 34` queries outside operation-mode query `11`
  - status: open but outside the supported surface
- Any hidden independent live keyboard-state getter in unknown service/admin command families
  - status: not found in the currently known surface; treated as unsupported rather than actively expected

## Resolved

### Operation Mode `cmd 30` Return Code

- old assumption: `u32::MAX` might be a success code
- resolved conclusion:
  - `cmd 30` return codes are non-authoritative
  - success must be verified through `cmd 34/query 11`

### Fan Live Mode Probe

- old assumption: `cmd 10/query 512` might be the authoritative fan mode source
- resolved conclusion:
  - supported fan mode comes from the validated combined model
  - `cmd 10/query 512` is diagnostic only

### Keyboard Live Zone Getter

- old assumption: `cmd 12` might be a trustworthy live per-zone on/off getter
- resolved conclusion:
  - `cmd 12` is not trustworthy on this machine for mixed zone states
  - supported keyboard state is XML-backed

### Admin-Session Behavior

- old assumption: all admin-backed features might be session-scoped in the same way
- resolved conclusion:
  - `keyboard sticky` is session-scoped
  - DTS sound and WhisperMode are effectively shared/global on this machine

### Wired 3.5 mm Sound Path

- old assumption: a wired headset might require a separate Waves backend
- resolved conclusion:
  - the validated Realtek 3.5 mm output exposes the full NitroSense TrueHarmony surface
  - RESense reads and writes it through the same DTS admin commands as internal speakers
  - Bluetooth/non-DTS output remains outside the supported surface
