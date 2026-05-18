# Final Functionality TODO

This file tracks the remaining functionality-first work after the main subsystem validation pass.

Scope rules:

- Finish correctness and compatibility work before CLI polish, output redesign, UX cleanup, or profile features.
- Exclude the deferred wired `3.5 mm` Waves path until real hardware is available.
- Prefer exact source-of-truth documentation and repeatable validation over speculative behavior.

## Current Baseline

- Fan: complete for the supported surface
- Keyboard: complete for the supported XML-backed surface
- Display: complete for LCD overdrive
- Operation mode: complete for the supported surface
- Sound: complete for the DTS/internal-speaker path
- Deferred hardware path: wired `3.5 mm` Waves sound validation

## Remaining Functionality Work

### 1. Service, Pipes, and Sessions

- [x] Verify session-dependent admin-agent behavior for sticky keys.
  Verified on 2026-05-18 with two active Windows sessions:
  - current `Donmez` session id / admin pipe: `4`
  - second `resense` session id / admin pipe: `5`
  - direct write to `PredatorSense_admin_agent_5` changed sticky keys only in the `resense` session
  - normal `resense keyboard sticky enable` from session `4` changed the current `Donmez` live sticky-key state only
  - `HKLM\SOFTWARE\OEM\NitroSense\AdvanceSettings\StickyKey` is not a reliable cross-session truth source by itself
- [x] Verify session-dependent admin-agent behavior for DTS sound getter/setter.
  Verified on 2026-05-18 with two active Windows sessions:
  - direct write to `PredatorSense_admin_agent_5` changed NitroSense sound preset in both sessions
  - normal `resense sound music` from session `4` also changed NitroSense sound preset in both sessions
  - current conclusion: DTS sound preset control is transported through the session admin pipe but the controlled state is effectively shared/global on this machine
- [x] Verify session-dependent admin-agent behavior for WhisperMode.
  Verified on 2026-05-18 with two active Windows sessions after restart:
  - current `Donmez` session / admin pipe: `1`
  - second `resense` session / admin pipe: `2`
  - direct write to `PredatorSense_admin_agent_2` changed NVIDIA WhisperMode to `On (60 FPS)` in both sessions while NitroSense mode stayed `Default`
  - direct write to `PredatorSense_admin_agent_1` changed NVIDIA WhisperMode back to `Off` in both sessions
  - current conclusion: WhisperMode control is transported through the session admin pipe but the controlled NVIDIA state is effectively shared/global on this machine
- [x] Confirm whether multiple active sessions can change which admin pipe should be targeted.
  Verified on 2026-05-18:
  - with both `PredatorSense_admin_agent_4` and `PredatorSense_admin_agent_5` alive, explicit writes to each pipe affected different sessions
  - admin-backed features are therefore genuinely session-scoped and can affect the wrong user session if the wrong pipe is chosen
- [x] Decide whether "first working session wins" is the correct supported contract or whether session targeting must become explicit.
  Current conclusion:
  - session-scoped admin features must target the current session explicitly
  - effectively global/shared admin-backed features may use the first reachable admin pipe
  - RESense now applies that split:
    - `keyboard sticky` -> current session only
    - `WhisperMode` and DTS sound -> first reachable admin pipe remains acceptable on this machine
- [x] Record the exact failure mode when:
  - the current session has no admin pipe
  - a different session owns the only working admin pipe
  - multiple session pipes are present at once
  Current conclusion:
  - `keyboard sticky` now fails safely if the current session admin pipe is unavailable instead of falling through to another user session
  - WhisperMode and DTS sound remain allowed to use another reachable admin pipe because their controlled state is effectively shared/global on this machine

### 2. Verification and Transparency

- [x] For every supported command, write down:
  - command
  - expected service writes
  - expected registry changes
  - expected XML changes
  - expected live readback
  - expected NitroSense-visible result
- [x] Keep a dedicated unresolved-assumptions log and explicitly move items out of it only when validated.

### 3. Deferred Hardware Validation

- [ ] Validate the wired `3.5 mm` Waves sound path later with a real supported device.

## Recommended Order

1. Deferred wired `3.5 mm` Waves validation when hardware is available
