# Project Roadmap

The current Windows AN515-58 functionality is complete. This file tracks
future product work.

## Current Status

Completed and supported:

- Fan telemetry, global auto, max, custom per-fan control, remembered manual
  percentages, and quiet-mode policy.
- Quiet, default, and performance modes with verified readback and the
  environment-dependent WhisperMode integration.
- Keyboard brightness, four-zone static lighting, dynamic effects, direction,
  speed, color, timeout, Sticky Keys, and Windows/Menu lock.
- LCD overdrive read/write with verified readback.
- DTS sound presets for internal speakers and wired 3.5 mm Realtek output.
- Session-aware settings and shared system settings follow the behavior of the
  Acer software.
- Stable status reads, typed device APIs, registry access, and atomic keyboard
  profile persistence.
- Formatting, warnings-as-errors Clippy, all-target tests, release builds,
  and the three-cycle hardware matrix with restoration.

## Supported Scope

- Sound presets use the DTS path for built-in speakers and wired 3.5 mm Realtek
  headphones.
- Keyboard lighting is persisted through the NitroSense keyboard profile.
- Windows and Acer Nitro AN515-58 are the implemented platform and model.

## Future Functionality

### Profiles

Profiles should be declarative desired-state documents, not trigger containers.
The lowest-friction first operation is saving the current state; programmatic
creation, editing, import/export, full profiles, and subsystem profiles can
follow. Profile fields must use the typed `Device` API, be stored
as inspectable JSON in user-owned app data, and be applied through the same
validated writes and readback verification as direct commands. Missing fields
leave current state unchanged. Contradictory combinations, such as quiet mode
with manual fan control, should be rejected rather than silently merged.

### Automations And External Hooks

Automation is separate from a profile: a profile describes state, while an
automation describes triggers and validated RESense actions. The first useful
actions are profile application, direct feature actions, and ordered action
sequences. The initial engine should execute only validated RESense
operations, not arbitrary shell commands. Process, power, headphone, and
session events are reasonable first triggers; complex expression trees and
thermal rule engines should wait.

External notifications or scripts may be added later as explicit opt-in hooks,
kept separate from first-party hardware writes. They should not enlarge the
core device API or silently run arbitrary commands.

### Linux And Other Backends

Linux requires a native hardware integration. The Linux research notes preserve
the EC register leads, keyboard-device leads, and validation order in
[`docs/re/linux-port.md`](re/linux-port.md).
Fan control, operation modes, RPM, and temperatures are the strongest initial
targets; keyboard readback and LCD overdrive remain unproven.

Other deferred work includes broader Acer model support, battery charge
limits, fan curves, telemetry export, GUI/tray integration, and remote
control. None should be added speculatively.

## Acceptance Bar

Any new hardware surface must have:

1. A typed read model and typed write path.
2. Verified readback and a documented state model.
3. Explicit unsupported behavior where the hardware cannot provide the
   requested state.
4. Code-level tests plus reversible hardware validation when hardware access
   is required.

Keep future abstractions grounded in a concrete second backend or product use
case.
