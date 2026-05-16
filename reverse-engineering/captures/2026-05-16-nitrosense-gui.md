# NitroSense Live GUI Capture - 2026-05-16

This capture was taken while NitroSense was open after access returned with KB5089549. The extraction was non-invasive: no UI controls were clicked or changed.

## Process And Package

- Process: `NitroSense.exe`
- PID at capture time: `23300`
- Window title: `NitroSense`
- Start time: `2026-05-16 16:56:12`
- Package: `AcerIncorporated.NitroSenseV31_3.1.3052.0_x64__48frkmn4z8aw4`
- Version: `3.1.3052.0`
- Install path: `C:\Program Files\WindowsApps\AcerIncorporated.NitroSenseV31_3.1.3052.0_x64__48frkmn4z8aw4`

## Active Named Pipes

- `\\.\pipe\PredatorSense_admin_agent_2`
- `\\.\pipe\predatorsense_service_namedpipe`

The service pipe appeared in lowercase during enumeration. Existing RESense code uses `\\.\pipe\PredatorSense_service_namedpipe`, which still works on this system.

## UI Automation Structure

Top-level window:

- AutomationId: `mainWindow`
- Class: `Window`
- Name: `NitroSense`

Major visible/control classes found:

- `Nitro_FanControlPage`
- `Nitro_MonitoringPage_2022`
- `Popup_Lighting`
- `LightingDynamicUI`

Top navigation/control buttons:

| AutomationId | Name |
| --- | --- |
| `SoundMode_Button` | `Sound Mode` |
| `Lighting_Button` | `Keyboard Lighting` |
| `Setting_Button` | `Setting` |
| `Planet9_Button` | `Planet9` |
| `gfe_Button` | `NVidia GeForce Experience` |
| `Close_Button` | `Close NitroSense` |
| `Minimize_Button` | `Minimize NitroSense` |

## Fan UI State

| AutomationId | UI name | State |
| --- | --- | --- |
| `FanModeAuto` | `Auto` | `On` |
| `FanModeMax` | `Max` | `Off` |
| `FanModeCustom` | `Custom` | `Off` |
| `CPU_Auto` | `Oto` | `On` |
| `GPU1_Auto` | `Oto` | `On` |
| `CPU_ScrollBar` | CPU slider | disabled, value `5`, range `0..10` |
| `GPU1_ScrollBar` | GPU slider | disabled, value `5`, range `0..10` |
| `CPU_FanRate` | CPU fan RPM | `2727` at focused capture |
| `GPU1_FanRate` | GPU fan RPM | `3157` at focused capture |
| `ShowCoolBoosterStatusicon` | CoolBoost toggle | `Off` |

RESense status at the same session reported:

- `persisted_mode`: `auto`
- `exact_mode_detail`: `global_auto`
- CPU/GPU custom auto flags: `true` / `true`
- CPU/GPU custom percentages: `50` / `50`

This matches the NitroSense fan UI state.

## Monitoring UI State

Point-in-time values from the focused UI Automation capture:

| AutomationId | Value |
| --- | --- |
| `CPU_Frequency_Text` | `2811` MHz |
| `GPU_Frequency_Text` | `210` MHz |
| `CPU_Templature` | `91°` |
| `GPU_Templature` | `46°` |
| `CPU_Usage` | `23` |
| `GPU_Usage` | `0` |
| `CPU_MinTemplature_value` | `53°` |
| `CPU_MaxTemplature_value` | `91°` |
| `GPU_MinTemplature_value` | `46°` |
| `GPU_MaxTemplature_value` | `48°` |

RESense status in the same session reported CPU temperature `89`, GPU temperature `46`, CPU fan `2586`, and GPU fan `2884`. The values are close enough for live telemetry sampled at different instants.

## Operation Mode UI State

| AutomationId | UI name | State |
| --- | --- | --- |
| `OPMode_Quiet_RadioButton` | `Sessiz` | not selected |
| `OPMode_Default_RadioButton` | `Varsayılan` | selected |
| `OPMode_Extreme_RadioButton` | `Performans` | not selected |

Related warning text:

- `WaringOverlocking`: `Mod değiştirme yalnızca AC adaptörü takılı olduğunda ve en az %40 pil ömrü kaldığında kullanılabilir.`

RESense status matched:

- live mode: `default`
- live mode code: `1`
- persisted operation mode code: `1`

## Keyboard Lighting UI State

| AutomationId | UI name | State |
| --- | --- | --- |
| `static_RadioButton` | `Statik` | selected |
| `dynamic_RadioButton` | `Dinamik` | not selected |
| `Brightness_ScrollBar` | brightness | value `2`, range `1..5` |
| `Lighting_Zone1_Checkbox` | `Zone 1 Lighting` | `On` |
| `Lighting_Zone2_Checkbox` | `Zone 2 Lighting` | `On` |
| `Lighting_Zone3_Checkbox` | `Zone 3 Lighting` | `On` |
| `Lighting_Zone4_Checkbox` | `Zone 4 Lighting` | `On` |

ProgramData XML matched the UI:

- Profile XML: `C:\ProgramData\OEM\NitroSense\ProfilePool\LightProfilePool\Default\Main.xml`
- `<Key status="0" brightness="2">`
- `<LightingEffects brightness="2">`
- Zone 1..4 status: `1`
- Zone 1..4 color: `#FF69B4`

Hidden/stored dynamic controls were also present:

| AutomationId | UI name | State |
| --- | --- | --- |
| `Light_Effects_RadioButton0` | `Nefes alma` | not selected |
| `Light_Effects_RadioButton1` | `Dalga` | not selected |
| `Light_Effects_RadioButton2` | `Yakınlaştır` | not selected |
| `Light_Effects_RadioButton3` | `Kayma` | selected |
| `Light_Effects_RadioButton4` | `Neon` | not selected |
| `Speed_ScrollBar` | speed | value `3`, range `1..9` |
| `Direction4way1` | `Direction Right` | selected |
| `Direction4way2` | `Direction Left` | not selected |

The XML stored dynamic state also had `<Pattern selected="3" color="#00FF00">`, matching `Kayma` / shifting as the stored dynamic mode while static lighting was active.

## Registry Snapshot Highlights

Root:

- `Model_Name_1st`: `nitro an515-58`
- `Model_Name_2nd`: `jimny_adh`
- `RGB_Keyboard_Support`: `1`
- `DTS_Audio_Support`: `1`
- `WHISPERMODE2.0_Support`: `0`

Fan:

- `CurrentFanMode`: `0`
- `CPUFanCustomAuto`: `1`
- `GPU1FanCustomAuto`: `1`
- `CPUFanPercentage`: `0x32` / `50`
- `GPU1FanPercentage`: `0x32` / `50`
- `CoolBoostMode`: `0`

Operation mode:

- `CurrentOperationMode`: `1`

Advanced settings:

- `LCD_Overdrive_support`: `1`
- `StickyKey`: `0`
- `WinAndMenuKey`: `1`

Lighting:

- `KeyBoardArea`: `4`
- `KeyBoardColor`: `2`
- `LightingProfile`: `Default`

Recent colors:

- `recent_color1`: `#3C1A38`
- `recent_color2`: `#352036`
- `recent_color3`: `#3C2949`

## New Finding: Zone Getter Needs Re-Verification

NitroSense UI and XML both reported all four lighting zones enabled, but RESense status reported every `live_zone_status` as disabled.

Direct command `12` reads returned:

| Zone | Query value | Raw reply | Decoded u64 | Low byte | Current RESense decode |
| --- | ---: | --- | --- | ---: | --- |
| 1 | `1` | `01080000000100000000000000` | `1` | `1` | `false` |
| 2 | `2` | `01080000000100000000000000` | `1` | `1` | `false` |
| 3 | `4` | `01080000000100000000000000` | `1` | `1` | `false` |
| 4 | `8` | `01080000000100000000000000` | `1` | `1` | `false` |

Current RESense code treats `(value & 0xFF) == 0` as enabled. With the restored NitroSense GUI open, the evidence points to either an inverted decode or a more nuanced meaning for command `12`. This should be re-tested by toggling one zone in NitroSense and comparing the command `12` value before changing RESense behavior.

## NitroSense GUI Refresh Behavior

The running NitroSense WPF UI does not generally watch the HKLM/XML state that RESense updates. Decompilation showed most pages read persisted state during construction or page-specific initialization, which explains why closing and reopening NitroSense shows RESense changes.

Useful `Nitro_MainWindow_Octavia.WndProc` messages found:

| Message | NitroSense behavior | RESense use |
| ---: | --- | --- |
| `32772` | calls `load_advance_settings(needset: true)` | best-effort refresh after advanced setting writes |
| `32783` | sets `Lighting_popup_page.Brightness_ScrollBar.Value = wParam` when RGB keyboard support is active | best-effort live brightness update after `resense keyboard brightness` |
| `32784` | updates visible WhisperMode status text from `wParam` | best-effort text refresh after operation-mode writes |

RESense intentionally does not drive the running NitroSense UI. Early live-refresh experiments showed that fan controls require NitroSense page click handlers and can leave the GUI in an inconsistent visual state when manipulated through UI Automation. The healthier workflow is to apply changes through RESense, then close and reopen NitroSense when the GUI needs to show the updated persisted values.

## Follow-Up Targets

- Re-test service command `12` by toggling one NitroSense zone at a time.
- Confirm whether dynamic speed really has a UI range of `1..9`; RESense currently documents and exposes `1..5`.
- Capture Sound Mode and Settings popups separately if UI navigation is acceptable.
- Use a pipe monitor or targeted before/after snapshots while changing one NitroSense control at a time.
