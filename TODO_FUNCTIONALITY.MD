# Functionality Completion

This file is the functional correctness gate for RESense.

The goal is simple:

- Read every supported feature reliably.
- Write every supported feature reliably.
- Know exactly which source is live, persisted, or both.
- Remove ambiguous states such as `auto_like`, `unknown`, or "best effort" behavior wherever the feature is considered supported.
- Document every remaining limitation explicitly until it is resolved.

No major CLI redesign, UX cleanup, or new features should outrank this file until the items below are complete enough that RESense can honestly describe its behavior as fully understood and dependable.

## Rules

- [x] Every supported read path must declare whether it is live, persisted, or a validated combination of both.
- [x] Every supported write path must have a verified readback path.
- [x] Every state mapping must be fully decoded. No `-like` buckets for supported functionality.
- [x] Every write command must define what it changes in hardware, service state, registry state, XML state, and NitroSense-visible state.
- [x] Every fallback or inferred behavior must either be fully validated or removed.
- [x] Every unsupported or not-yet-decoded behavior must be called out plainly in code and docs.
- [x] `reverse-engineering/README.MD` must stay aligned with the actual code and current confidence level.

# Cross-Cutting Work

- [x] Create a feature-by-feature source-of-truth matrix:
  - feature
  - live read path
  - persisted read path
  - write path
  - verification method
  - current confidence
  - unresolved questions
- [x] Define what "fully supported" means for each subsystem and apply that standard consistently.
- [x] Replace vague reliability wording with concrete wording tied to known commands, payloads, registry values, XML nodes, and observed hardware behavior.
- [x] Add repeatable verification procedures for each command so future changes can be re-validated quickly.
- [x] Record raw request and reply payloads for all supported read and write operations.
- [x] Identify every place where code currently infers state instead of reading it directly.
- [x] Identify every place where code writes persisted state as a synchronization step and verify whether that persisted state is actually authoritative, merely NitroSense-facing, or just a cache.

# Fan

- [x] Fully define the supported fan read model without depending on `auto_like`.
  RESense now treats exact active fan mode as a validated combined read:
  - live service command `13` for RPM and temperature
  - `HKLM\SOFTWARE\OEM\NitroSense\FanControl` for exact active mode
  - `CurrentFanMode` is the authoritative high-level mode
  - per-fan custom fields are only active truth when `CurrentFanMode=custom`
- [x] Remove unsupported diagnostic fan-mode-byte decoding from the supported fan model.
  `cmd 10/query 512` is no longer part of the supported fan read path because NitroSense itself does not use it as the exact fan-state source. RESense now exposes only the validated model it actually relies on.
- [x] Verify whether global auto, custom-all-auto, CPU-manual/GPU-auto, CPU-auto/GPU-manual, and CPU-manual/GPU-manual each have unique live readback signatures.
- [x] Verify whether registry fan state always matches actual applied service state after each write.
  Verified on 2026-05-17 for:
  - `global_auto`
  - `max`
  - `custom_all_auto`
  - `custom_cpu_manual`
  - `custom_gpu_manual`
  - `custom_cpu_gpu_manual`
  Caveat:
  - remembered custom fields stay populated outside custom mode, so they must not be treated as active truth unless `CurrentFanMode=custom`
- [x] Verify whether the service write alone is sufficient, or whether registry sync is required for correctness, NitroSense compatibility, or both.
  Current conclusion:
  - service writes control the hardware behavior
  - registry sync is required for NitroSense-visible state and for exact state restoration in the NitroSense UI model
- [x] Verify whether fan percentages are persisted only, live-readable, or reconstructable only through combined sources.
  Current conclusion:
  - exact percentages come from NitroSense registry state
  - RPM is the live hardware effect
  - the service mode probe does not expose exact custom percentages
- [x] Confirm all edge cases around quiet mode fan locking and ensure readback reflects the enforced state exactly.
  Verified on 2026-05-17:
  - entering `resense mode quiet` from a manual custom fan state forced NitroSense into global `Auto`
  - NitroSense visually locked fan control to auto while quiet mode was active
  - RESense readback showed `mode.live_mode=quiet` and fan `exact_mode_detail=global_auto`
  - `CurrentFanMode=0` after the transition
  - `fan mode max` was blocked by RESense while quiet mode remained active
- [x] Remove any remaining ambiguous fan notes from the supported fan read path.
  The supported fan path no longer exposes `-like`, `partial`, or diagnostic-only ambiguity.

# Keyboard

- [x] Keep keyboard backlight timeout under the keyboard subsystem, not the display subsystem.
  Current conclusion:
  - the feature controls keyboard lighting behavior, not LCD behavior
  - the command surface now lives under `keyboard`
  - live status for the timeout toggle is reported with keyboard state
- [x] Validate keyboard backlight timeout as a supported enable/disable feature.
  Verified on 2026-05-17:
  - NitroSense and RESense stay aligned on the timeout toggle
  - the live getter reports timeout `0` or `30` correctly
  - keyboard brightness remains owned by the `keyboard brightness` subsystem
- [x] Decide whether a true live readback for keyboard brightness exists in the currently known service surface.
  Current conclusion:
  - no trustworthy independent live brightness getter was found in the currently known service/query surface on this machine
  - controlled raw sweeps across command families `10`, `12`, `20`, and `34` did not change when the physical keyboard moved from full brightness to NitroSense's darkest visible state
  - the supported keyboard brightness read model therefore remains the validated NitroSense XML state
- [x] Validate the current supported keyboard mode/brightness/color state model even if no independent live getter exists.
  Current conclusion:
  - NitroSense XML is currently the strongest supported keyboard state source on this machine
  - validated against NitroSense and the physical keyboard for:
    - static vs dynamic mode
    - brightness
    - dynamic effect
    - dynamic direction
    - dynamic speed
    - dynamic color for effects that visibly use color
    - remembered static zones while dynamic mode is active
- [x] Decide whether a true live readback for keyboard static/dynamic mode exists in the currently known service surface.
  Current conclusion:
  - no trustworthy independent live static/dynamic mode getter was found in the currently known service/query surface on this machine
  - the supported current-mode read model remains the validated NitroSense XML state
- [x] Decide whether a true live readback for keyboard RGB colors exists in the currently known service surface.
  Current conclusion:
  - no trustworthy independent live RGB color getter was found in the currently known service/query surface on this machine
  - NitroSense XML remains the supported source for current keyboard colors
- [x] Define NitroSense XML's current role in the supported keyboard model.
  Current conclusion:
  - managed NitroSense keyboard UI restores static/dynamic mode, brightness, zone status, zone colors, dynamic effect, dynamic speed, dynamic direction, and pattern color from `LightingProfileXML`
  - the same managed UI then pushes commands `27`, `28`, and `29` from that XML-backed state
  - there are no managed GUI call sites for `cmd 12`
  - controlled raw sweeps across the currently known getter families (`10`, `12`, `20`, `34`) did not reveal a separate trustworthy live keyboard-state getter
  - for the currently supported keyboard surface on this machine, NitroSense XML is the strongest state source and behaves as NitroSense's own effective state model
- [x] Verify whether `Key.status`, `Key.brightness`, `LightingEffects.brightness`, `Pattern.selected`, `Pattern.color`, and per-zone XML values each reflect live state, persisted state, or stale state.
  Current conclusion:
  - `LightingEffects.brightness` tracked the effective NitroSense slider / visible keyboard brightness in the validated static and dynamic cases
  - `Key.brightness` can remain stale and is not the effective slider truth by itself
  - `Key.status` and `Pattern.selected` tracked static vs dynamic mode in the validated cases
  - `Pattern.color`, per-pattern speed/direction, and per-zone XML values tracked the validated NitroSense-visible current state
  - these XML values should currently be treated as NitroSense's effective persisted state model, not as an independently proven live getter
- [x] Decide whether any true live zone enable/disable readback exists in the currently known service surface for all four zones and mixed on/off combinations.
  Current conclusion:
  - `service cmd 12` is not a trustworthy live per-zone on/off source on this machine
  - NitroSense and the physical keyboard showed a mixed `1/0/1/0` static pattern while all four `cmd 12` replies remained identical
  - wider sweeps of the currently known getter families did not reveal another trustworthy live zone-state getter
- [x] Confirm that static writes change only the requested zones and do not unintentionally alter unrelated zones, brightness, or supported remembered state in the validated cases.
  Current conclusion:
  - validated partial static writes changed only the requested zones
  - unrelated zone colors/statuses and brightness stayed untouched in the validated cases
  - remembered dynamic metadata remains outside the currently supported keyboard read contract and is not surfaced as independent live truth
- [x] Confirm that partial static writes leave unrelated zones and brightness untouched in the validated cases.
  Verified on 2026-05-17:
  - `resense keyboard static --zone1 0000FF`
  - only zone 1 changed
  - zones 2, 3, and 4 remained unchanged
  - brightness remained unchanged
- [x] Verify that static writes can leave dynamic mode cleanly and produce the intended NitroSense-visible zone pattern.
  Verified on 2026-05-17:
  - `resense keyboard static --zone1 FF0000 --zone2 off --zone3 00FF00 --zone4 off`
  - NitroSense switched from dynamic back to static
  - the physical keyboard matched the exact `red/off/green/off` pattern
  - XML readback matched the same final state
- [x] Confirm that dynamic writes map exactly to the supported service/XML/hardware behavior for the validated dynamic modes and parameters.
  Current conclusion:
  - validated dynamic writes covered real effect transitions plus direction, speed, and color changes
  - NitroSense UI, XML state, and the physical keyboard remained aligned in the validated cases
  - the supported contract is the validated dynamic surface already exercised, not an exhaustive proof of every unreachable or visually indistinguishable combination
- [x] Verify that dynamic writes and XML sync match NitroSense-visible state for at least one real effect transition and one parameter change.
  Verified on 2026-05-17:
  - `resense keyboard dynamic wave --direction from-left` flipped NitroSense and hardware from right-to-left back to left-to-right
  - `resense keyboard dynamic breathing` switched NitroSense and hardware from wave to breathing
  - XML stayed aligned for mode, brightness, speed, color, and direction
- [x] Verify that keyboard brightness writes match NitroSense and the physical keyboard.
  Verified on 2026-05-17:
  - `resense keyboard brightness 5`
  - NitroSense slider moved to full brightness
  - the physical keyboard returned to full brightness
- [x] Verify that at least one dynamic effect uses RESense-written color and speed on real hardware.
  Verified on 2026-05-17:
  - `resense keyboard dynamic breathing --speed 9 --color 00FFFF`
  - NitroSense switched to `Nefes alma`
  - the physical keyboard changed clearly to cyan
  - the breathing speed became visibly fast
- [x] Remove inferred dynamic defaults as a hidden behavior unless they are intentionally part of the product contract and clearly documented.
  Current conclusion:
  - missing dynamic speed/color/direction values are intentionally resolved from the current validated XML-backed keyboard state, then from explicit defaults only when no prior supported state exists
  - this behavior is now part of the documented supported keyboard contract
- [x] Verify whether command `27`, `28`, and `29` responsibilities are completely understood for the supported keyboard surface.
  Current conclusion:
  - `27` controls keyboard backlight mode/brightness/dynamic effect payloads
  - `28` sets per-zone RGB for enabled static zones
  - `29` sets per-zone enabled/disabled behavior
  - no additional managed NitroSense keyboard service command was found for the currently supported surface
- [x] Verify how NitroSense represents keyboard state when zones are off but last colors are preserved.
  Current conclusion:
  - when a zone is turned off, NitroSense XML preserves the last zone color while setting `status=0`
  - RESense mirrors that behavior so the remembered color survives re-enabling
- [x] Define exactly what "current keyboard state" means for the supported surface on this machine.
  Current conclusion:
  - the supported current keyboard state is NitroSense's effective XML-backed state model, validated against NitroSense UI and physical keyboard behavior
  - RESense does not claim a separate independent live keyboard-state getter on this machine because none has been validated in the currently known service/query surface

# Display

- [x] Narrow the display subsystem to LCD overdrive only.
  Current conclusion:
  - keyboard backlight timeout no longer belongs to `display`
  - display support now means LCD overdrive capability and live state
- [x] Verify whether overdrive support and overdrive enabled state are fully and reliably readable on the supported machine.
  Verified on 2026-05-17 on this AN515-58:
  - `LCD_Overdrive_support=1` is a capability flag, not current state
  - NitroSense off matched `overdrive_live=false`
  - RESense `display overdrive enable` flipped NitroSense back on visibly
- [x] Finish the supported display contract and remove unsupported timeout-related caveats from it.
  The supported display surface is now the validated LCD overdrive feature only.

# Operation Mode

- [x] Fully decode the supported operation mode mapping.
  Verified on 2026-05-17:
  - `quiet` -> `mode_code=0`
  - `default` -> `mode_code=1`
  - `performance` -> `mode_code=4`
  - service getter `status` remained `0` in all validated visible NitroSense modes
- [x] Verify whether the service getter is sufficient to represent the real applied mode in the supported visible mode set.
  Verified on 2026-05-17:
  - NitroSense `Default`, `Performance`, and `Quiet` all matched `service cmd 34/query 11`
- [x] Verify whether the registry mode code is authoritative, merely persisted UI state, or only a compatibility mirror.
  Current conclusion:
  - `HKLM\SOFTWARE\OEM\NitroSense\Overclock\CurrentOperationMode` mirrors the selected NitroSense mode code correctly
  - the live source of truth remains `service cmd 34/query 11`
- [x] Confirm that each supported mode write produces exactly the intended final NitroSense-visible state.
  Verified on 2026-05-17:
  - RESense writes for `default`, `performance`, and `quiet` all updated NitroSense visibly
  - `quiet` still enforced the already-validated fan `global_auto` side effect
- [x] Decide whether WhisperMode failure should remain best-effort or become explicit user-visible behavior.
  Current conclusion:
  - on this machine, WhisperMode visibility depends on the NVIDIA environment
  - RESense should keep treating WhisperMode as best-effort and non-authoritative for the core mode-state model
- [x] Verify the exact role of WhisperMode integration during quiet mode transitions on a machine/driver state where NVIDIA exposes a meaningful WhisperMode signal.
  Verified on 2026-05-17 in hybrid mode with NVIDIA App:
  - starting from NitroSense `default`, NVIDIA `WhisperMode` showed `Off`
  - `resense mode quiet` switched NitroSense to `Quiet`
  - NVIDIA `WhisperMode` changed from `Off` to an active `60 FPS` setting
  - this confirms the admin-agent WhisperMode side effect is real when the NVIDIA stack exposes the feature
- [x] Validate the real role of operation-mode return code `u32::MAX` and update the supported contract accordingly.
  Current conclusion:
  - `cmd 30` return codes are not authoritative enough to represent success or failure by themselves on this machine
  - the same raw reply / `u32::MAX` return code was observed for:
    - real mode changes (`default -> quiet`, `quiet -> default`)
    - a no-op (`default -> default`)
    - at least one write that initially did not change the live mode (`default -> performance`)
  - NitroSense's own managed UI ignores the numeric return from `set_operation_mode()` and updates UI/registry before firing the service write
  - the supported RESense contract is now:
    - send `cmd 30`
    - verify the final live mode through `cmd 34/query 11`
    - sync `CurrentOperationMode` only after live verification succeeds

# Sound

- [x] Narrow the currently supported sound surface to the validated DTS/internal-speaker path until a supported wired-headphone path is decoded.
  Current conclusion:
  - RESense now treats sound support as DTS-only
  - Bluetooth headphone output is not part of the supported sound surface
  - Waves validation is deferred until a supported `3.5 mm` device is available
- [x] Finish the supported DTS mode map without leaving user-visible `unknown` states in the supported surface.
  Verified on 2026-05-17 for the DTS-backed `Acer TrueHarmony` surface with headphones disconnected:
  - `Otomatik` -> `auto` / `10`
  - `Müzik` -> `music` / `0`
  - `Film` -> `movies` / `1`
  - `Ses` -> `voice` / `2`
  - `Strateji` -> `strategy` / `3`
  - `RPG` -> `rpg` / `4`
  - `Nişancı` -> `shooter` / `5`
  - `Özel Ses` -> `custom` / `6`
- [x] Verify whether backend auto-resolution is always correct and whether registry support flags are sufficient for the supported sound path.
  Current conclusion:
  - `DTS_Audio_Support` is sufficient for the currently supported DTS/internal-speaker path
  - when the default output changed to Bluetooth headphones, the backend no longer stayed on the validated DTS path
  - that non-DTS state is now treated as unsupported rather than as an active supported fallback
- [x] Verify whether headphones or other device state alter sound command behavior in ways the CLI must report explicitly.
  Verified on 2026-05-17:
  - Bluetooth headphones changed the NitroSense preset surface and the backend no longer matched the validated DTS path
  - NitroSense notes this surface is intended for internal speakers and `3.5 mm` audio devices
  - the CLI should therefore treat Bluetooth/non-DTS audio state as unsupported for now
- [x] Confirm that write verification after preset changes is complete and reliable for the supported DTS path.
  Verified on 2026-05-17 for DTS writes:
  - `resense sound music` -> NitroSense `Müzik`
  - `resense sound auto` -> NitroSense `Otomatik`
  - `resense sound shooter` -> NitroSense `Nişancı`
- [ ] Validate the supported wired-headphone Waves path later with a real `3.5 mm` device and document its exact preset surface.

# Service, Pipes, and Sessions

- [x] Catalog every named-pipe command currently used by RESense with exact payload encoding and decoded reply structure.
- [x] Identify any commands whose reply bytes are still only partially understood.
- [ ] Verify session-dependent admin-agent behavior for sticky keys, WhisperMode, and sound operations.
- [ ] Confirm whether there are cases where multiple active sessions change which admin pipe should be targeted.
- [x] Document failure modes for stopped, missing, or unhealthy service/admin-agent states.

# Verification and Transparency

- [ ] For every supported command, write down:
  - command
  - expected service writes
  - expected registry changes
  - expected XML changes
  - expected live readback
  - expected NitroSense-visible result
- [x] Add a dedicated validation checklist for one-setting-at-a-time NitroSense comparison.
- [ ] Keep a log of unresolved assumptions and do not silently treat them as settled behavior.
- [x] Remove or rename any code path, field name, or output label that implies certainty where only inference exists.
- [x] Once a feature is fully decoded, tighten code comments and output wording so they state exact behavior, not tentative behavior.

# Exit Criteria

- [x] No supported feature depends on an ambiguous `-like` state.
- [x] No supported feature depends on undocumented inferred behavior.
- [x] Every supported write has exact, verified readback.
- [x] Every supported read clearly states and proves its source.
- [x] Docs, code, and observed hardware behavior all agree for the supported surface.
