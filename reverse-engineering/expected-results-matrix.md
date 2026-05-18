# RESense Expected Results Matrix

This file defines the expected outcome of every supported RESense command on the validated surface.

Conventions:

- "service write" means a write on `\\.\pipe\PredatorSense_service_namedpipe`
- "admin write" means a write on `\\.\pipe\PredatorSense_admin_agent_<session_id>`
- "HKLM sync" means a NitroSense-facing compatibility mirror, not always the live source of truth
- "XML sync" means NitroSense keyboard profile state under `C:\ProgramData\OEM\NitroSense`

## Fan

| Command | Expected service write(s) | Expected registry change(s) | Expected XML change(s) | Expected live readback | Expected NitroSense-visible result |
| --- | --- | --- | --- | --- | --- |
| `resense fan mode auto` | `cmd 15` global-auto payload | `FanControl\\CurrentFanMode=0`; remembered custom fields preserved | none | `fan.exact_mode_detail=global_auto`; RPM/temp still from `cmd 13` | fan page shows global `Auto` |
| `resense fan mode max` | `cmd 15` max payload | `FanControl\\CurrentFanMode=1` | none | `fan.exact_mode_detail=max` | fan page shows `Max` |
| `resense fan speed --cpu-auto --gpu-auto` | `cmd 15` custom-all-auto payload | `FanControl\\CurrentFanMode=2`; both custom-auto flags `1` | none | `fan.exact_mode_detail=custom_all_auto` | fan page stays in `Custom`, both fans `Auto` |
| `resense fan speed --cpu 70 --gpu-auto` | `cmd 15` custom behavior plus `cmd 16` CPU percent | `FanControl\\CurrentFanMode=2`; CPU manual at `70`; GPU auto | none | `fan.exact_mode_detail=custom_cpu_manual` | `Custom`, CPU manual, GPU `Auto` |
| `resense fan speed --cpu-auto --gpu 70` | `cmd 15` custom behavior plus `cmd 16` GPU percent | `FanControl\\CurrentFanMode=2`; CPU auto; GPU manual at `70` | none | `fan.exact_mode_detail=custom_gpu_manual` | `Custom`, CPU `Auto`, GPU manual |
| `resense fan speed --cpu 70 --gpu 70` | `cmd 15` custom behavior plus `cmd 16` CPU/GPU percent | `FanControl\\CurrentFanMode=2`; CPU manual; GPU manual | none | `fan.exact_mode_detail=custom_cpu_gpu_manual` | `Custom`, both fans manual |

## Keyboard

| Command | Expected service write(s) | Expected registry change(s) | Expected XML change(s) | Expected live readback | Expected NitroSense-visible result |
| --- | --- | --- | --- | --- | --- |
| `resense keyboard brightness <1..5>` | `cmd 27` brightness payload | none | `Main.xml`: `LightingEffects.brightness`; `Key.brightness` mirror | `keyboard.state.brightness` from XML | NitroSense slider and physical keyboard brightness change |
| `resense keyboard static ...` | `cmd 27` brightness/static-mode transition; `cmd 29` zone behavior; `cmd 28` enabled-zone RGB | none | `Main.xml`: `Key.status=0`; `LightingEffects_Zone*`; brightness preserved | `keyboard.state.mode=static`; static zones from XML | NitroSense static page and keyboard zones match |
| `resense keyboard dynamic <mode> ...` | `cmd 27` dynamic payload | none | `Main.xml`: `Key.status=1`; `Pattern.selected`; selected pattern `speed/direction`; `Pattern.color`; brightness preserved | `keyboard.state.mode=dynamic`; dynamic metadata from XML | NitroSense dynamic page and keyboard effect match |
| `resense keyboard backlight-timeout enable` | `cmd 17` timeout payload | none | none | `keyboard.backlight_timeout_live=true` from `cmd 20` | NitroSense advanced toggle on; keyboard turns off after timeout |
| `resense keyboard backlight-timeout disable` | `cmd 17` timeout payload | none | none | `keyboard.backlight_timeout_live=false` from `cmd 20` | NitroSense advanced toggle off |
| `resense keyboard sticky enable` | admin `cmd 2` on current session pipe only | `AdvanceSettings\\StickyKey=1` compatibility sync | none | current-session `SystemParametersInfo` sticky state becomes enabled | current-session NitroSense sticky toggle follows current session |
| `resense keyboard sticky disable` | admin `cmd 2` on current session pipe only | `AdvanceSettings\\StickyKey=0` compatibility sync | none | current-session `SystemParametersInfo` sticky state becomes disabled | current-session NitroSense sticky toggle follows current session |
| `resense keyboard win-menu enable` | `cmd 9` selector `2` | `AdvanceSettings\\WinAndMenuKey=1` | none | `keyboard.win_menu_key_lock_live=true` from `cmd 10/query 0` | NitroSense advanced toggle on |
| `resense keyboard win-menu disable` | `cmd 9` selector `2` | `AdvanceSettings\\WinAndMenuKey=0` | none | `keyboard.win_menu_key_lock_live=false` from `cmd 10/query 0` | NitroSense advanced toggle off |

## Operation Mode

| Command | Expected service write(s) | Expected registry change(s) | Expected XML change(s) | Expected live readback | Expected NitroSense-visible result |
| --- | --- | --- | --- | --- | --- |
| `resense mode default` | `cmd 30` mode code `1`; optional admin `cmd 15` WhisperMode off | `Overclock\\CurrentOperationMode=1` after verified success | none | `mode.live_mode.mode=default`; `mode_code=1` from `cmd 34/query 11` | NitroSense `Default` selected |
| `resense mode performance` | `cmd 30` mode code `4`; optional admin `cmd 15` WhisperMode off | `Overclock\\CurrentOperationMode=4` after verified success | none | `mode.live_mode.mode=performance`; `mode_code=4` | NitroSense `Performance` selected |
| `resense mode quiet` | `cmd 30` mode code `0`; optional admin `cmd 15` WhisperMode on | `Overclock\\CurrentOperationMode=0` after verified success; fan registry forced to global auto by mode behavior | none | `mode.live_mode.mode=quiet`; `mode_code=0` | NitroSense `Quiet` selected; fan page locked to auto; NVIDIA WhisperMode on when exposed |

## Display

| Command | Expected service write(s) | Expected registry change(s) | Expected XML change(s) | Expected live readback | Expected NitroSense-visible result |
| --- | --- | --- | --- | --- | --- |
| `resense display overdrive enable` | `cmd 9` selector `0x10` with enable bit | none required for current-state truth; capability bit remains in HKLM | none | `display.state.overdrive_live=true` | NitroSense advanced `LCD Overdrive` on |
| `resense display overdrive disable` | `cmd 9` selector `0x10` with disable state | none | none | `display.state.overdrive_live=false` | NitroSense advanced `LCD Overdrive` off |

## Sound

| Command | Expected service write(s) | Expected registry change(s) | Expected XML change(s) | Expected live readback | Expected NitroSense-visible result |
| --- | --- | --- | --- | --- | --- |
| `resense sound auto` | admin `cmd 14` preset `10` | none required beyond `DTS_Audio_Support` support gate | none | admin `cmd 13` returns `10` on the supported DTS path | NitroSense sound page `Otomatik` |
| `resense sound music` | admin `cmd 14` preset `0` | none | none | admin `cmd 13` returns `0` on the supported DTS path | NitroSense sound page `Müzik` |
| `resense sound movies` | admin `cmd 14` preset `1` | none | none | admin `cmd 13` returns `1` | NitroSense sound page `Film` |
| `resense sound voice` | admin `cmd 14` preset `2` | none | none | admin `cmd 13` returns `2` | NitroSense sound page `Ses` |
| `resense sound strategy` | admin `cmd 14` preset `3` | none | none | admin `cmd 13` returns `3` | NitroSense sound page `Strateji` |
| `resense sound rpg` | admin `cmd 14` preset `4` | none | none | admin `cmd 13` returns `4` | NitroSense sound page `RPG` |
| `resense sound shooter` | admin `cmd 14` preset `5` | none | none | admin `cmd 13` returns `5` | NitroSense sound page `Nişancı` |
| `resense sound custom` | admin `cmd 14` preset `6` | none | none | admin `cmd 13` returns `6` | NitroSense sound page `Özel Ses` |

## Notes

- Keyboard state remains XML-backed by design on the supported surface.
- Sticky keys are current-session only.
- DTS sound and WhisperMode are effectively shared/global on this machine even though they are transported through session admin pipes.
- The wired `3.5 mm` Waves path is intentionally outside this matrix until it is validated.
