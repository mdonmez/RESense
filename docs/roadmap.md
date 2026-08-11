# Project Roadmap

The functionality-first refactor is complete for the validated Windows
AN515-58 surface. This is the canonical project-status and future-work file;
the old root TODO and design drafts were consolidated here.

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
- Current-session targeting for session-scoped settings and discovered-session
  routing for shared admin-backed settings.
- Strict status reads, minimal stable JSON, typed pipe transport, `winreg`
  reads, atomic keyboard XML persistence, and the feature-gated developer
  probe.
- Formatting, warnings-as-errors Clippy, all-target tests, release builds,
  and the three-cycle hardware matrix with restoration.

## Explicit Boundaries

- Bluetooth and other non-DTS sound paths are known unsupported; they are not
  treated as DTS or routed through an unvalidated fallback.
- NitroSense XML is the supported keyboard source of truth where the vendor
  surface does not provide a validated independent live getter.
- Unknown protocol/query values fail explicitly instead of becoming
  `unknown`, `partial`, or `-like` production states.
- Windows and Acer Nitro AN515-58 are the only implemented platform/model
  combination.

## Future Functionality

### Profiles

Profiles should be declarative desired-state documents, not trigger containers.
The lowest-friction first operation is saving the current state; programmatic
creation, editing, import/export, full profiles, and partial subsystem
profiles can follow. Profile fields must use the typed `Device` API, be stored
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

Linux is a backend rewrite, not a compatibility layer over NitroSense. The
Linux research notes preserve the EC register leads, keyboard-device leads,
source-of-truth questions, and validation order in
[`docs/re/linux-port.md`](re/linux-port.md).
Fan control, operation modes, RPM, and temperatures are the strongest initial
targets; keyboard readback and LCD overdrive remain unproven.

Other deferred work includes broader Acer model support, battery charge
limits, fan curves, telemetry export, GUI/tray integration, and remote
control. None should be added speculatively.

## Acceptance Bar

Any new hardware surface must have:

1. A typed read model and typed write path.
2. Verified readback and a documented source of truth.
3. Explicit unsupported behavior where the hardware cannot provide the
   requested state.
4. Code-level tests plus reversible hardware validation when hardware access
   is required.

Do not add a generic backend or action-engine abstraction until a second real
backend or a concrete profile/automation use case justifies it.
