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

The service endpoint was enumerated case-insensitively.

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

RESense reported global automatic fan control with both fan controls set to automatic, matching the NitroSense fan UI state.

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

## Keyboard Zone Observation

NitroSense UI and XML reported all four lighting zones enabled. The captured vendor zone response returned low byte `1` for each zone. The device layer reads zone state from the keyboard profile XML.

Direct command `12` reads returned:

| Zone | Query value | Response low byte | Profile state |
| --- | ---: | ---: | --- |
| 1 | `1` | `1` | enabled |
| 2 | `2` | `1` | enabled |
| 3 | `4` | `1` | enabled |
| 4 | `8` | `1` | enabled |

The captured response and profile state agree for the all-zones-enabled case. This observation is retained as research context; the current device layer uses the keyboard profile for zone state.

## NitroSense GUI Refresh Behavior

The running NitroSense WPF UI does not generally watch the HKLM/XML state that RESense updates. Decompilation showed most pages read persisted state during construction or page-specific initialization, which explains why closing and reopening NitroSense shows RESense changes.

NitroSense reads persisted page state when the relevant page initializes. Close and reopen the page when the UI needs to display a change made externally.
