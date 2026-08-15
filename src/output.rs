use crate::cli::StatusTarget;
use crate::device::{
    Direction, DynamicEffect, DynamicLighting, FanCustomControl, FanMode, FanState, KeyboardState,
    LightingMode, OperationMode, Rgb, SoundPreset, SystemState,
};
use crate::error::Result;
use serde::Serialize;
use std::fmt::Write as _;

pub enum StatusValue {
    All(SystemState),
    Fan(FanState),
    Keyboard(KeyboardState),
    Mode(OperationMode),
    Display(Option<bool>),
    Sound(Option<SoundPreset>),
}

#[derive(Debug, Serialize)]
struct StatusJson {
    fan: FanJson,
    keyboard: KeyboardJson,
    mode: OperationMode,
    display: Option<bool>,
    sound: Option<SoundPreset>,
}

#[derive(Debug, Serialize)]
struct FanJson {
    mode: FanMode,
    cpu: FanTelemetryJson,
    gpu: FanTelemetryJson,
    custom: FanCustomJson,
}

#[derive(Debug, Serialize)]
struct FanTelemetryJson {
    temperature_c: u16,
    rpm: u16,
}

#[derive(Debug, Serialize)]
struct FanCustomJson {
    cpu: FanCustomControlJson,
    gpu: FanCustomControlJson,
}

#[derive(Debug, Serialize)]
struct FanCustomControlJson {
    mode: &'static str,
    percent: u8,
}

#[derive(Debug, Serialize)]
struct KeyboardJson {
    brightness: u8,
    lighting: LightingJson,
    backlight_timeout: bool,
    sticky_keys: bool,
    win_menu_lock: bool,
}

#[derive(Debug, Serialize)]
struct LightingJson {
    mode: &'static str,
    #[serde(rename = "static")]
    static_config: StaticLightingJson,
    dynamic: DynamicLightingJson,
}

#[derive(Debug, Serialize)]
struct StaticLightingJson {
    zones: [ZoneJson; 4],
}

#[derive(Debug, Serialize)]
struct DynamicLightingJson {
    effect: &'static str,
    color: Option<Rgb>,
    speed: u8,
    direction: Option<Direction>,
}

#[derive(Debug, Serialize)]
struct ZoneJson {
    enabled: bool,
    color: Rgb,
}

pub fn print_status(
    value: StatusValue,
    target: Option<StatusTarget>,
    json: bool,
    compact_json: bool,
) -> Result<()> {
    if json {
        if target.is_some() {
            print_json(&target_json(value, compact_json)?)
        } else {
            print_json(&all_json(value, compact_json)?)
        }
    } else {
        print_text(value);
        Ok(())
    }
}

fn all_json(value: StatusValue, compact: bool) -> Result<String> {
    match value {
        StatusValue::All(state) => json_string(&status_json(&state), compact),
        _ => bail_type("full status rendering received a targeted value"),
    }
}

fn target_json(value: StatusValue, compact: bool) -> Result<String> {
    match value {
        StatusValue::All(state) => json_string(&status_json(&state), compact),
        StatusValue::Fan(state) => json_string(&fan_json(&state), compact),
        StatusValue::Keyboard(state) => json_string(&keyboard_json(&state), compact),
        StatusValue::Mode(state) => json_string(&state, compact),
        StatusValue::Display(state) => json_string(&state, compact),
        StatusValue::Sound(state) => json_string(&state, compact),
    }
}

fn json_string<T: serde::Serialize>(value: &T, compact: bool) -> Result<String> {
    if compact {
        Ok(serde_json::to_string(value)?)
    } else {
        Ok(serde_json::to_string_pretty(value)?)
    }
}

fn print_json(value: &str) -> Result<()> {
    println!("{value}");
    Ok(())
}

fn print_text(value: StatusValue) {
    print!("{}", render_text(value));
}

fn render_text(value: StatusValue) -> String {
    let mut output = String::new();
    match value {
        StatusValue::All(state) => {
            render_fan(&mut output, &state.fan);
            render_keyboard(&mut output, &state.keyboard);
            writeln!(output, "mode={}", mode_name(state.mode)).unwrap();
            writeln!(
                output,
                "display.overdrive={}",
                optional_bool(state.display_overdrive)
            )
            .unwrap();
            writeln!(output, "sound={}", optional_sound(state.sound)).unwrap();
        }
        StatusValue::Fan(state) => render_fan(&mut output, &state),
        StatusValue::Keyboard(state) => render_keyboard(&mut output, &state),
        StatusValue::Mode(mode) => {
            writeln!(output, "mode={}", mode_name(mode)).unwrap();
        }
        StatusValue::Display(value) => {
            writeln!(output, "display.overdrive={}", optional_bool(value)).unwrap();
        }
        StatusValue::Sound(value) => {
            writeln!(output, "sound={}", optional_sound(value)).unwrap();
        }
    }
    output
}

pub fn print_fan_state(state: FanState) {
    print!("{}", render_fan_text(&state));
}

pub fn print_keyboard_state(state: KeyboardState) {
    print!("{}", render_keyboard_text(&state));
}

pub fn print_mode(mode: OperationMode) {
    println!("mode={}", mode_name(mode));
}

pub fn print_display(value: Option<bool>) {
    println!("display.overdrive={}", optional_bool(value));
}

pub fn print_sound(value: SoundPreset) {
    println!("sound={}", sound_name(value));
}

fn render_fan_text(state: &FanState) -> String {
    let mut output = String::new();
    render_fan(&mut output, state);
    output
}

fn render_fan(output: &mut String, state: &FanState) {
    writeln!(output, "fan.mode={}", fan_mode_name(state.mode)).unwrap();
    render_fan_telemetry(output, "fan.cpu", &state.cpu);
    render_fan_telemetry(output, "fan.gpu", &state.gpu);
    render_fan_custom(output, "fan.custom.cpu", state.custom.cpu);
    render_fan_custom(output, "fan.custom.gpu", state.custom.gpu);
}

fn render_fan_telemetry(output: &mut String, prefix: &str, channel: &crate::device::FanTelemetry) {
    writeln!(output, "{prefix}.temperature_c={}", channel.temperature_c).unwrap();
    writeln!(output, "{prefix}.rpm={}", channel.rpm).unwrap();
}

fn render_fan_custom(output: &mut String, prefix: &str, control: FanCustomControl) {
    writeln!(output, "{prefix}.mode={}", control.mode_name()).unwrap();
    writeln!(output, "{prefix}.percent={}", control.percent().get()).unwrap();
}

fn render_keyboard_text(state: &KeyboardState) -> String {
    let mut output = String::new();
    render_keyboard(&mut output, state);
    output
}

fn render_keyboard(output: &mut String, state: &KeyboardState) {
    writeln!(output, "keyboard.brightness={}", state.brightness.get()).unwrap();
    writeln!(
        output,
        "keyboard.lighting.mode={}",
        lighting_mode_name(state.lighting.mode)
    )
    .unwrap();
    render_zones(
        output,
        "keyboard.lighting.static",
        &state.lighting.static_zones,
    );
    render_dynamic_lighting(output, "keyboard.lighting.dynamic", state.lighting.dynamic);
    writeln!(
        output,
        "keyboard.backlight_timeout={}",
        state.backlight_timeout
    )
    .unwrap();
    writeln!(output, "keyboard.sticky_keys={}", state.sticky_keys).unwrap();
    writeln!(output, "keyboard.win_menu_lock={}", state.win_menu_lock).unwrap();
}

fn render_zones(output: &mut String, prefix: &str, zones: &[crate::device::Zone; 4]) {
    for (index, zone) in zones.iter().enumerate() {
        writeln!(
            output,
            "{prefix}.zone{}.enabled={}",
            index + 1,
            zone.enabled
        )
        .unwrap();
        writeln!(output, "{prefix}.zone{}.color={}", index + 1, zone.color).unwrap();
    }
}

fn render_dynamic_lighting(output: &mut String, prefix: &str, lighting: DynamicLighting) {
    let (name, color, direction) = dynamic_effect_text(lighting.effect);
    writeln!(output, "{prefix}.effect={name}").unwrap();
    writeln!(
        output,
        "{prefix}.color={}",
        color
            .map(|value| value.to_string())
            .unwrap_or_else(|| "null".to_string())
    )
    .unwrap();
    writeln!(output, "{prefix}.speed={}", lighting.speed.get()).unwrap();
    writeln!(output, "{prefix}.direction={}", direction.unwrap_or("null")).unwrap();
}

fn status_json(state: &SystemState) -> StatusJson {
    StatusJson {
        fan: fan_json(&state.fan),
        keyboard: keyboard_json(&state.keyboard),
        mode: state.mode,
        display: state.display_overdrive,
        sound: state.sound,
    }
}

fn fan_json(state: &FanState) -> FanJson {
    FanJson {
        mode: state.mode,
        cpu: fan_telemetry_json(&state.cpu),
        gpu: fan_telemetry_json(&state.gpu),
        custom: FanCustomJson {
            cpu: fan_custom_control_json(state.custom.cpu),
            gpu: fan_custom_control_json(state.custom.gpu),
        },
    }
}

fn fan_telemetry_json(channel: &crate::device::FanTelemetry) -> FanTelemetryJson {
    FanTelemetryJson {
        temperature_c: channel.temperature_c,
        rpm: channel.rpm,
    }
}

fn fan_custom_control_json(control: FanCustomControl) -> FanCustomControlJson {
    match control {
        FanCustomControl::Auto { percent } => FanCustomControlJson {
            mode: "auto",
            percent: percent.get(),
        },
        FanCustomControl::Manual { percent } => FanCustomControlJson {
            mode: "manual",
            percent: percent.get(),
        },
    }
}

fn keyboard_json(state: &KeyboardState) -> KeyboardJson {
    KeyboardJson {
        brightness: state.brightness.get(),
        lighting: LightingJson {
            mode: lighting_mode_name(state.lighting.mode),
            static_config: StaticLightingJson {
                zones: state.lighting.static_zones.map(|zone| ZoneJson {
                    enabled: zone.enabled,
                    color: zone.color,
                }),
            },
            dynamic: dynamic_lighting_json(state.lighting.dynamic),
        },
        backlight_timeout: state.backlight_timeout,
        sticky_keys: state.sticky_keys,
        win_menu_lock: state.win_menu_lock,
    }
}

fn dynamic_lighting_json(lighting: DynamicLighting) -> DynamicLightingJson {
    let (effect, color, direction) = dynamic_effect_json(lighting.effect);
    DynamicLightingJson {
        effect,
        color,
        speed: lighting.speed.get(),
        direction,
    }
}

fn dynamic_effect_json(effect: DynamicEffect) -> (&'static str, Option<Rgb>, Option<Direction>) {
    match effect {
        DynamicEffect::Breathing { color } => ("breathing", Some(color), None),
        DynamicEffect::Neon => ("neon", None, None),
        DynamicEffect::Shifting { color, direction } => ("shifting", Some(color), Some(direction)),
        DynamicEffect::Wave { color, direction } => ("wave", Some(color), Some(direction)),
        DynamicEffect::Zoom { color } => ("zoom", Some(color), None),
    }
}

fn dynamic_effect_text(effect: DynamicEffect) -> (&'static str, Option<Rgb>, Option<&'static str>) {
    match effect {
        DynamicEffect::Breathing { color } => ("breathing", Some(color), None),
        DynamicEffect::Neon => ("neon", None, None),
        DynamicEffect::Shifting { color, direction } => {
            ("shifting", Some(color), Some(direction_name(direction)))
        }
        DynamicEffect::Wave { color, direction } => {
            ("wave", Some(color), Some(direction_name(direction)))
        }
        DynamicEffect::Zoom { color } => ("zoom", Some(color), None),
    }
}

fn direction_name(direction: Direction) -> &'static str {
    match direction {
        Direction::FromLeft => "fromleft",
        Direction::FromRight => "fromright",
    }
}

fn mode_name(mode: OperationMode) -> &'static str {
    match mode {
        OperationMode::Quiet => "quiet",
        OperationMode::Default => "default",
        OperationMode::Performance => "performance",
    }
}

fn fan_mode_name(mode: FanMode) -> &'static str {
    match mode {
        FanMode::Auto => "auto",
        FanMode::Max => "max",
        FanMode::Custom => "custom",
    }
}

fn lighting_mode_name(mode: LightingMode) -> &'static str {
    match mode {
        LightingMode::Static => "static",
        LightingMode::Dynamic => "dynamic",
    }
}

fn sound_name(sound: SoundPreset) -> &'static str {
    match sound {
        SoundPreset::Music => "music",
        SoundPreset::Movies => "movies",
        SoundPreset::Voice => "voice",
        SoundPreset::Strategy => "strategy",
        SoundPreset::Rpg => "rpg",
        SoundPreset::Shooter => "shooter",
        SoundPreset::Custom => "custom",
        SoundPreset::Auto => "auto",
    }
}

fn optional_bool(value: Option<bool>) -> &'static str {
    match value {
        Some(true) => "true",
        Some(false) => "false",
        None => "null",
    }
}

fn optional_sound(value: Option<SoundPreset>) -> &'static str {
    value.map(sound_name).unwrap_or("null")
}

fn bail_type<T>(message: &str) -> Result<T> {
    anyhow::bail!("{message}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::device::{
        Brightness, DynamicSpeed, FanCustomControl, FanCustomState, FanTelemetry,
        KeyboardLightingState, Percent, Zone,
    };

    fn sample_state() -> SystemState {
        SystemState {
            fan: FanState {
                mode: FanMode::Custom,
                cpu: FanTelemetry {
                    temperature_c: 60,
                    rpm: 2400,
                },
                gpu: FanTelemetry {
                    temperature_c: 45,
                    rpm: 2300,
                },
                custom: FanCustomState {
                    cpu: FanCustomControl::Manual {
                        percent: Percent::new(70).unwrap(),
                    },
                    gpu: FanCustomControl::Auto {
                        percent: Percent::new(50).unwrap(),
                    },
                },
            },
            keyboard: KeyboardState {
                brightness: Brightness::new(5).unwrap(),
                lighting: KeyboardLightingState {
                    mode: LightingMode::Static,
                    static_zones: [
                        Zone {
                            enabled: true,
                            color: Rgb::parse("FF0000").unwrap(),
                        },
                        Zone {
                            enabled: false,
                            color: Rgb::parse("00FF00").unwrap(),
                        },
                        Zone {
                            enabled: true,
                            color: Rgb::parse("0000FF").unwrap(),
                        },
                        Zone {
                            enabled: false,
                            color: Rgb::parse("FFFFFF").unwrap(),
                        },
                    ],
                    dynamic: DynamicLighting {
                        effect: DynamicEffect::Wave {
                            color: Rgb::parse("00FFFF").unwrap(),
                            direction: Direction::FromLeft,
                        },
                        speed: DynamicSpeed::new(3).unwrap(),
                    },
                },
                backlight_timeout: true,
                sticky_keys: false,
                win_menu_lock: true,
            },
            mode: OperationMode::Performance,
            display_overdrive: Some(true),
            sound: Some(SoundPreset::Music),
        }
    }

    #[test]
    fn fan_text_uses_stable_selector_and_custom_paths() {
        let output = render_text(StatusValue::Fan(sample_state().fan));
        assert_eq!(
            output,
            "fan.mode=custom\n\
fan.cpu.temperature_c=60\n\
fan.cpu.rpm=2400\n\
fan.gpu.temperature_c=45\n\
fan.gpu.rpm=2300\n\
fan.custom.cpu.mode=manual\n\
fan.custom.cpu.percent=70\n\
fan.custom.gpu.mode=auto\n\
fan.custom.gpu.percent=50\n"
        );
    }

    #[test]
    fn keyboard_text_contains_both_lighting_blocks() {
        let output = render_text(StatusValue::Keyboard(sample_state().keyboard));
        assert!(output.contains("keyboard.lighting.mode=static\n"));
        assert!(output.contains("keyboard.lighting.static.zone1.enabled=true\n"));
        assert!(output.contains("keyboard.lighting.static.zone4.color=#FFFFFF\n"));
        assert!(output.contains("keyboard.lighting.dynamic.effect=wave\n"));
        assert!(output.contains("keyboard.lighting.dynamic.color=#00FFFF\n"));
        assert!(output.contains("keyboard.lighting.dynamic.speed=3\n"));
        assert!(output.contains("keyboard.lighting.dynamic.direction=fromleft\n"));
        assert!(!output.contains("keyboard.lighting.zone1."));
    }

    #[test]
    fn full_json_uses_stable_mode_scoped_paths() {
        let value = serde_json::from_str::<serde_json::Value>(
            &all_json(StatusValue::All(sample_state()), false).unwrap(),
        )
        .unwrap();
        assert_eq!(value["fan"]["custom"]["cpu"]["mode"], "manual");
        assert_eq!(value["fan"]["custom"]["cpu"]["percent"], 70);
        assert_eq!(value["fan"]["custom"]["gpu"]["mode"], "auto");
        assert_eq!(value["fan"]["custom"]["gpu"]["percent"], 50);
        assert!(value["fan"]["cpu"].get("control").is_none());
        assert!(value["fan"]["cpu"].get("effective_control").is_none());
        assert_eq!(
            value["keyboard"]["lighting"]["static"]["zones"]
                .as_array()
                .unwrap()
                .len(),
            4
        );
        assert_eq!(value["keyboard"]["lighting"]["dynamic"]["effect"], "wave");
        assert_eq!(
            value["keyboard"]["lighting"]["dynamic"]["direction"],
            "from_left"
        );
        assert!(value["keyboard"]["lighting"].get("zones").is_none());
        assert_eq!(value["mode"], "performance");
        assert!(value.get("trust").is_none());
        assert!(value.get("reliability").is_none());
        assert!(value.get("source").is_none());
        assert!(value.get("mode_code").is_none());
    }

    #[test]
    fn targeted_json_is_not_wrapped() {
        let mode = serde_json::from_str::<serde_json::Value>(
            &target_json(StatusValue::Mode(OperationMode::Quiet), false).unwrap(),
        )
        .unwrap();
        assert_eq!(mode, serde_json::json!("quiet"));

        let display = serde_json::from_str::<serde_json::Value>(
            &target_json(StatusValue::Display(None), false).unwrap(),
        )
        .unwrap();
        assert!(display.is_null());
    }

    #[test]
    fn compact_json_is_single_line_for_watch_mode() {
        let value = all_json(StatusValue::All(sample_state()), true).unwrap();
        assert!(!value.contains('\n'));
        assert!(value.starts_with('{'));
    }

    #[test]
    fn selector_explains_which_fan_block_is_applied() {
        let mut state = sample_state();
        state.fan.mode = FanMode::Auto;

        let value = serde_json::from_str::<serde_json::Value>(
            &all_json(StatusValue::All(state), false).unwrap(),
        )
        .unwrap();
        assert_eq!(value["fan"]["mode"], "auto");
        assert_eq!(value["fan"]["custom"]["cpu"]["mode"], "manual");
        assert_eq!(value["fan"]["custom"]["cpu"]["percent"], 70);
        assert_eq!(value["fan"]["custom"]["gpu"]["mode"], "auto");
    }
}
