# RESense Source-of-Truth Matrix

This file defines the supported RESense surface feature by feature.

The standard used here:

- `live`: direct service/admin/OS state readback.
- `persisted`: registry or XML state only.
- `validated combination`: more than one source is intentionally combined and re-verified against NitroSense and hardware.
- `fully supported`: the current read source, write path, and NitroSense-visible result are all known well enough to be part of the supported contract on this machine.

## Feature Matrix

| Feature | Read model | Write path | NitroSense-visible sync | Verification method | Confidence | Unresolved questions |
| --- | --- | --- | --- | --- | --- | --- |
| Fan RPM and temperatures | `live`, service `cmd 13`, queries `0x0101`, `0x0201`, `0x0601`, `0x0A01` | none | not needed | compared against NitroSense RPM/temp widgets | validated | none for supported surface |
| Fan active mode | `validated combination`: live RPM/temp from `cmd 13` plus HKLM `FanControl\\CurrentFanMode` and per-fan custom fields | service `cmd 15` and `cmd 16` plus registry sync | yes, registry sync is required | NitroSense fan page plus registry plus hardware RPM behavior | validated | none for supported surface |
| Fan remembered custom sliders | `persisted`, HKLM `FanControl` | service writes plus registry sync | yes | NitroSense custom fan page and registry snapshots | validated | remembered custom state outside custom mode is not active truth |
| Keyboard current mode | `persisted`, NitroSense `Main.xml` `Key.status` / `Pattern.selected`, validated against hardware | service `cmd 27/28/29` plus XML sync | yes | NitroSense keyboard page plus physical keyboard | validated | no independent live getter found |
| Keyboard brightness | `persisted`, NitroSense `Main.xml` `LightingEffects.brightness`, validated against hardware | service `cmd 27` plus XML sync | yes | NitroSense slider plus physical keyboard | validated | no independent live getter found |
| Keyboard static zones | `persisted`, NitroSense `Main.xml` `LightingEffects_Zone*`, validated against hardware | service `cmd 28/29` plus XML sync | yes | NitroSense static page plus physical keyboard | validated | no independent live getter found |
| Keyboard dynamic effect/direction/speed/color | `persisted`, NitroSense `Main.xml` `Pattern`, validated against hardware | service `cmd 27` plus XML sync | yes | NitroSense dynamic page plus physical keyboard | validated | no independent live getter found |
| Keyboard backlight timeout | `live`, service `cmd 20` with `BK_Hotkey_Number` payload | service `cmd 17` with `BK_Hotkey_Number` payload | yes | NitroSense advanced settings plus timeout behavior | validated | extra brightness byte is intentionally outside supported contract |
| Sticky keys | `live`, Windows `SystemParametersInfo` | admin pipe `cmd 2`; HKLM compatibility sync is NitroSense-facing only and not a reliable cross-session truth source | partial: NitroSense advanced setting follows HKLM | Windows state plus NitroSense toggle | live | WhisperMode and sound multi-session behavior still worth documenting |
| Win/Menu key lock | `live`, service `cmd 10/query 0` bit decode | service `cmd 9` selector `2` plus HKLM compatibility sync | yes | NitroSense advanced setting plus live readback | live/validated | none for supported surface |
| Operation mode | `live`, service `cmd 34/query 11`; HKLM mode code is a mirror | service `cmd 30` plus live verification via `cmd 34/query 11`; WhisperMode best-effort side effect over admin pipe and effectively shared/global across active sessions on this machine; HKLM sync after verified success | yes | NitroSense mode cards, NVIDIA App for WhisperMode | validated | none for supported surface |
| LCD overdrive | `live` enabled state from service `cmd 10/query 0`; support flag from HKLM `LCD_Overdrive_support` | service `cmd 9` selector `0x10` | yes | NitroSense advanced settings | validated | none for supported surface |
| Sound preset (supported path) | `live`, admin pipe `cmd 13` on DTS/internal-speaker path; HKLM `DTS_Audio_Support` gates support | admin pipe `cmd 14`; validated as effectively shared/global across active sessions on this machine | yes | NitroSense sound page on internal speakers | validated | wired `3.5 mm` Waves path deferred; multi-session WhisperMode still open |

## Inference / Fallback Inventory

These behaviors are intentional and part of the current contract:

| Location | Behavior | Why it exists | Current status |
| --- | --- | --- | --- |
| `keyboard dynamic` | missing speed/color/direction are filled from current validated XML state, then defaults if no prior state exists | preserve NitroSense-visible state when user changes only one dynamic parameter | documented supported behavior |
| Fan active mode | combines live thermals/RPM with registry mode state | service surface does not provide exact active fan mode alone | documented supported behavior |
| Sound backend `auto` | resolves only to DTS for the supported surface | non-DTS paths are not yet validated | documented supported behavior |

## Persisted-State Sync Inventory

These writes intentionally update persisted state after hardware/service writes:

| Feature | Persisted target | Why sync is kept |
| --- | --- | --- |
| Fan mode / fan custom state | HKLM `FanControl` | NitroSense uses it as the exact active/remembered fan UI model |
| Keyboard mode/brightness/zones/pattern | NitroSense `Main.xml` | NitroSense keyboard UI is XML-first and restores state from it |
| Sticky keys | HKLM `AdvanceSettings\\StickyKey` | NitroSense-facing compatibility mirror |
| Win/Menu key lock | HKLM `AdvanceSettings\\WinAndMenuKey` | NitroSense-facing compatibility mirror |
| Operation mode | HKLM `Overclock\\CurrentOperationMode` | NitroSense-facing compatibility mirror |

## Unsupported / Deferred

These are explicitly outside the current supported surface:

- independent live keyboard-state getter for mode/brightness/colors/zones
- `cmd 12` as a live keyboard zone-state source
- wired `3.5 mm` Waves sound path until real hardware is available
- any Bluetooth/non-DTS sound path
