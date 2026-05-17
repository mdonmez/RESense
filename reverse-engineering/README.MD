# RESense Reverse Engineering Notes

## Current Context

NitroSense stopped launching after the 2026-04 Windows update cycle on this AN515-58, which forced RESense to become the practical control surface for fan, keyboard, mode, display, and sound features.

As of 2026-05-15, after KB5089549, NitroSense is accessible again on this machine. RESense development continues, but the restored GUI now gives a stronger live reverse-engineering path: NitroSense can be used as the reference UI while RESense observes and validates the same behavior.

## Captures

- `captures/2026-05-16-nitrosense-gui.md` - first non-invasive capture from the restored NitroSense GUI using process inspection, UI Automation, registry/XML snapshots, named-pipe enumeration, and `resense status`.

## Working Docs

- `source-of-truth-matrix.md` - supported feature-by-feature read/write/source matrix.
- `pipe-command-catalog.md` - currently used service/admin command catalog with payload and reply notes.
- `validation-checklist.md` - repeatable one-setting-at-a-time re-validation workflow.

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
- Fan active mode is now treated as a validated combined read model, not as a single live getter:
  - live source: service command `13` for temperatures and RPM values
  - exact active mode source: `HKLM\SOFTWARE\OEM\NitroSense\FanControl`
  - NitroSense itself initializes the fan UI from `CurrentFanMode`, `CPUFanPercentage`, `GPU1FanPercentage`, `CPUFanCustomAuto`, and `GPU1FanCustomAuto`
  - those per-fan custom fields are only active truth when `CurrentFanMode=2` (`custom`); outside custom mode they are remembered slider state
- Keyboard brightness, static/dynamic mode, and RGB colors are currently treated as NitroSense persisted XML state because no trusted live getter has been found.
- NitroSense keyboard brightness currently tracks `LightingEffects.brightness` for both static and dynamic observations. `Key.brightness` can remain stale and should not be treated as the slider truth by itself.
- Keyboard zone enabled/disabled state does not currently have a trustworthy validated live getter on this machine:
  - `service cmd 12` returned identical replies for all four zones during a mixed NitroSense-visible static pattern
  - NitroSense and the physical keyboard showed `zone1=on`, `zone2=off`, `zone3=on`, `zone4=off`
  - all four raw replies still came back as `01080000000100000000000000`
  - RESense therefore no longer treats `cmd 12` as a supported live per-zone on/off source
- Wider raw sweeps across the currently known getter families also failed to reveal an independent keyboard-state getter:
  - a controlled baseline snapshot was taken at `static + all zones on + red + brightness 5`
  - a second snapshot was taken after changing only keyboard brightness to NitroSense's darkest visible state (`brightness=1`, physically dark)
  - command families `10`, `12`, `20`, and `34` were swept and compared between the two states
  - none of the successful raw replies changed with that visible hardware brightness transition
  - current conclusion: there is no validated independent live keyboard-state getter in the currently known service/query surface on this machine
- Managed NitroSense decompilation now supports the XML-first keyboard model:
  - `Popup_Lighting.load_lighting_profile()` restores static/dynamic mode, brightness, zone status, and zone colors from `LightingProfileXML`
  - `LightingDynamicUI.load_lighting_profile()` restores the dynamic effect, speed, direction, and pattern color from `LightingProfileXML`
  - `LightingDynamicUI.Set_Dynamic_Keyboard()` then pushes the current XML-backed dynamic state to command `27`
  - `Popup_Lighting.Set_Static_Keyboard_Set_Status()` / `Set_Static_Keyboard_ZoneButton_Color()` restore static UI state from XML and then push commands `27`, `28`, and `29`
  - there are no managed GUI call sites for `GetAcerGamingLEDGroupColor` (`cmd 12`)
  - the managed GUI uses `LightingEffects.brightness`, not `Key.brightness`, as the effective keyboard brightness source
- Backlight timeout is now treated as a validated keyboard-lighting feature:
  - service getter byte `[40..47]` confirms timeout `0` or `30`
  - RESense writes and NitroSense UI stayed aligned on the timeout toggle
  - the extra getter/setter brightness byte is real but is not part of the supported surface because it did not control immediate or post-timeout keyboard brightness on this machine
- NitroSense XML is currently the strongest keyboard state source on this machine:
  - static mode, dynamic mode, dynamic direction, dynamic brightness, and remembered static zones all tracked NitroSense UI changes correctly
  - `LightingEffects.brightness` tracked the effective visible brightness; `Key.brightness` can remain stale
  - validated writes and visible hardware behavior:
    - `resense keyboard brightness 5`
    - `resense keyboard dynamic wave --direction from-left`
    - `resense keyboard dynamic breathing`
    - `resense keyboard dynamic breathing --speed 9 --color 00FFFF`
    - `resense keyboard static --zone1 FF0000 --zone2 off --zone3 00FF00 --zone4 off`
    - `resense keyboard static --zone1 0000FF`
  - verified dynamic writes:
    - `resense keyboard dynamic wave --direction from-left`
    - `resense keyboard dynamic breathing`
  - verified static write leaving dynamic mode:
    - `resense keyboard static --zone1 FF0000 --zone2 off --zone3 00FF00 --zone4 off`
  - validated partial static write:
    - `resense keyboard static --zone1 0000FF`
    - only zone 1 changed; other zones and brightness stayed untouched
  - NitroSense preserves last zone colors even when a zone is off; RESense mirrors that XML behavior
- Operation mode, LCD Overdrive, and Windows/Menu lock have live readback paths.
- Supported operation mode mapping is now validated on this machine:
  - service command `34/query 11` matched NitroSense `Quiet`, `Default`, and `Performance`
  - validated mode codes are `0=quiet`, `1=default`, `4=performance`
  - `CurrentOperationMode` mirrored those values correctly in NitroSense registry state
  - in discrete-only mode / some NVIDIA states, WhisperMode may appear unavailable or inactive
  - in hybrid mode with NVIDIA App exposing WhisperMode, `resense mode quiet` changed WhisperMode from `Off` to an active `60 FPS` setting
  - WhisperMode therefore remains a real but environment-dependent side effect, not the primary source of truth for operation mode state
- DTS-backed sound mapping is now validated on this machine with headphones disconnected:
  - NitroSense `Acer TrueHarmony` matched the DTS backend
  - validated preset codes are:
    - `0=music`
    - `1=movies`
    - `2=voice`
    - `3=strategy`
    - `4=rpg`
    - `5=shooter`
    - `6=custom`
    - `10=auto`
  - verified CLI writes:
    - `resense sound music` -> NitroSense `Müzik`
    - `resense sound auto` -> NitroSense `Otomatik`
    - `resense sound shooter` -> NitroSense `Nişancı`
  - RESense now treats this DTS/internal-speaker path as the only supported sound surface
- Bluetooth headphone output changed the NitroSense menu to a different preset surface and no longer stayed on the validated DTS path:
  - RESense observed `backend=waves` and `dts_supported=false`
  - NitroSense clicks and RESense writes both left the live preset pinned at `music`
  - NitroSense indicates this surface is for internal speakers and `3.5 mm` audio devices
  - RESense therefore now treats Bluetooth/non-DTS output as unsupported instead of leaving an active Waves fallback in the supported code path
  - remaining sound gap is later validation of the wired `3.5 mm` Waves path

## Experiment Rules

- Keep each test narrow and repeatable.
- Prefer live hardware proof for fan behavior, especially RPM changes.
- Treat NitroSense XML and registry values as persisted UI state unless the test proves they match live hardware.
- Record return codes and raw payloads when a command succeeds with a non-obvious reply.
- Do not copy NitroSense UI restrictions into RESense unless they are required for hardware safety or correctness.
