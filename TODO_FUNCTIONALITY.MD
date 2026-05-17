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

- [ ] Every supported read path must declare whether it is live, persisted, or a validated combination of both.
- [ ] Every supported write path must have a verified readback path.
- [ ] Every state mapping must be fully decoded. No `-like` buckets for supported functionality.
- [ ] Every write command must define what it changes in hardware, service state, registry state, XML state, and NitroSense-visible state.
- [ ] Every fallback or inferred behavior must either be fully validated or removed.
- [ ] Every unsupported or not-yet-decoded behavior must be called out plainly in code and docs.
- [ ] `reverse-engineering/README.MD` must stay aligned with the actual code and current confidence level.

# Cross-Cutting Work

- [ ] Create a feature-by-feature source-of-truth matrix:
  - feature
  - live read path
  - persisted read path
  - write path
  - verification method
  - current confidence
  - unresolved questions
- [ ] Define what "fully supported" means for each subsystem and apply that standard consistently.
- [ ] Replace vague reliability wording with concrete wording tied to known commands, payloads, registry values, XML nodes, and observed hardware behavior.
- [ ] Add repeatable verification procedures for each command so future changes can be re-validated quickly.
- [ ] Record raw request and reply payloads for all supported read and write operations.
- [ ] Identify every place where code currently infers state instead of reading it directly.
- [ ] Identify every place where code writes persisted state as a synchronization step and verify whether that persisted state is actually authoritative, merely NitroSense-facing, or just a cache.

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
- [ ] Find and validate true live readback for keyboard brightness, if available.
- [ ] Find and validate true live readback for keyboard static/dynamic mode, if available.
- [ ] Find and validate true live readback for keyboard RGB colors, if available.
- [ ] Verify whether NitroSense XML is authoritative state, persisted mirror state, or only NitroSense UI state for each keyboard attribute.
- [ ] Verify whether `Key.status`, `Key.brightness`, `LightingEffects.brightness`, `Pattern.selected`, `Pattern.color`, and per-zone XML values each reflect live state, persisted state, or stale state.
- [ ] Confirm that zone enable/disable live readback is fully decoded for all four zones and mixed on/off combinations.
- [ ] Confirm that static writes change only the requested zones and do not unintentionally alter unrelated zones, brightness, or dynamic metadata.
- [ ] Confirm that dynamic writes map exactly to service behavior for every mode, speed, direction, and color combination.
- [ ] Remove inferred dynamic defaults as a hidden behavior unless they are intentionally part of the product contract and clearly documented.
- [ ] Verify whether command 27, 28, and 29 responsibilities are completely understood and whether any missing service command is still needed for full fidelity.
- [ ] Verify how NitroSense represents keyboard state when zones are off but last colors are preserved.
- [ ] Define exactly what "current keyboard state" means when live service state and XML state disagree.

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
- [ ] Validate whether return code `u32::MAX` is genuinely a success case and document why.

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

- [ ] Catalog every named-pipe command currently used by RESense with exact payload encoding and decoded reply structure.
- [ ] Identify any commands whose reply bytes are still only partially understood.
- [ ] Verify session-dependent admin-agent behavior for sticky keys, WhisperMode, and sound operations.
- [ ] Confirm whether there are cases where multiple active sessions change which admin pipe should be targeted.
- [ ] Document failure modes for stopped, missing, or unhealthy service/admin-agent states.

# Verification and Transparency

- [ ] For every supported command, write down:
  - command
  - expected service writes
  - expected registry changes
  - expected XML changes
  - expected live readback
  - expected NitroSense-visible result
- [ ] Add a dedicated validation checklist for one-setting-at-a-time NitroSense comparison.
- [ ] Keep a log of unresolved assumptions and do not silently treat them as settled behavior.
- [ ] Remove or rename any code path, field name, or output label that implies certainty where only inference exists.
- [ ] Once a feature is fully decoded, tighten code comments and output wording so they state exact behavior, not tentative behavior.

# Exit Criteria

- [ ] No supported feature depends on an ambiguous `-like` state.
- [ ] No supported feature depends on undocumented inferred behavior.
- [ ] Every supported write has exact, verified readback.
- [ ] Every supported read clearly states and proves its source.
- [ ] Docs, code, and observed hardware behavior all agree for the supported surface.
