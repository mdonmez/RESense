use crate::cli::StatusTarget;
use crate::device::{
    Direction, DynamicEffect, DynamicLighting, FanCustomControl, FanCustomRequest, FanMode,
    FanState, KeyboardState, LightingMode, OperationMode, Rgb, SoundPreset, SystemState,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    settings: Option<FanSettingsJson>,
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
struct FanSettingsJson {
    custom: FanCustomJson,
}

#[derive(Debug, Serialize)]
struct FanCustomControlJson {
    mode: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    percent: Option<u8>,
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
    settings: LightingSettingsJson,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
enum LightingSettingsJson {
    Static {
        zones: [ZoneJson; 4],
    },
    Breathing {
        color: Rgb,
        speed: u8,
    },
    Neon {
        speed: u8,
    },
    Shifting {
        color: Rgb,
        speed: u8,
        direction: Direction,
    },
    Wave {
        speed: u8,
        direction: Direction,
    },
    Zoom {
        color: Rgb,
        speed: u8,
    },
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

pub fn print_fan_mode(mode: FanMode) {
    print!("{}", render_fan_mode_text(mode));
}

pub fn print_fan_custom_state(state: FanState, request: FanCustomRequest) {
    print!("{}", render_fan_custom_text(&state, request));
}

pub fn print_keyboard_brightness(state: KeyboardState) {
    print!("{}", render_keyboard_brightness_text(&state));
}

pub fn print_keyboard_timeout(state: KeyboardState) {
    print!("{}", render_keyboard_timeout_text(&state));
}

pub fn print_keyboard_lighting(state: KeyboardState) {
    print!("{}", render_keyboard_lighting_text(&state));
}

pub fn print_keyboard_sticky_keys(state: KeyboardState) {
    print!("{}", render_keyboard_sticky_keys_text(&state));
}

pub fn print_keyboard_win_menu_lock(state: KeyboardState) {
    print!("{}", render_keyboard_win_menu_lock_text(&state));
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

fn render_fan_mode_text(mode: FanMode) -> String {
    format!("fan.mode={mode}\n", mode = fan_mode_name(mode))
}

fn render_fan_custom_text(state: &FanState, request: FanCustomRequest) -> String {
    let mut output = String::new();
    writeln!(output, "fan.mode={}", fan_mode_name(state.mode)).unwrap();
    if request.cpu().is_some() {
        render_fan_custom(&mut output, "fan.settings.custom.cpu", state.custom.cpu);
    }
    if request.gpu().is_some() {
        render_fan_custom(&mut output, "fan.settings.custom.gpu", state.custom.gpu);
    }
    output
}

fn render_fan(output: &mut String, state: &FanState) {
    writeln!(output, "fan.mode={}", fan_mode_name(state.mode)).unwrap();
    render_fan_telemetry(output, "fan.cpu", &state.cpu);
    render_fan_telemetry(output, "fan.gpu", &state.gpu);
    if state.mode == FanMode::Custom {
        render_fan_custom(output, "fan.settings.custom.cpu", state.custom.cpu);
        render_fan_custom(output, "fan.settings.custom.gpu", state.custom.gpu);
    }
}

fn render_fan_telemetry(output: &mut String, prefix: &str, channel: &crate::device::FanTelemetry) {
    writeln!(output, "{prefix}.temperature_c={}", channel.temperature_c).unwrap();
    writeln!(output, "{prefix}.rpm={}", channel.rpm).unwrap();
}

fn render_fan_custom(output: &mut String, prefix: &str, control: FanCustomControl) {
    writeln!(output, "{prefix}.mode={}", control.mode_name()).unwrap();
    if let FanCustomControl::Manual { percent } = control {
        writeln!(output, "{prefix}.percent={}", percent.get()).unwrap();
    }
}

fn render_keyboard_brightness_text(state: &KeyboardState) -> String {
    format!("keyboard.brightness={}\n", state.brightness.get())
}

fn render_keyboard_timeout_text(state: &KeyboardState) -> String {
    format!("keyboard.backlight_timeout={}\n", state.backlight_timeout)
}

fn render_keyboard_lighting_text(state: &KeyboardState) -> String {
    let mut output = String::new();
    render_keyboard_lighting(&mut output, state);
    output
}

fn render_keyboard_sticky_keys_text(state: &KeyboardState) -> String {
    format!("keyboard.sticky_keys={}\n", state.sticky_keys)
}

fn render_keyboard_win_menu_lock_text(state: &KeyboardState) -> String {
    format!("keyboard.win_menu_lock={}\n", state.win_menu_lock)
}

fn render_keyboard(output: &mut String, state: &KeyboardState) {
    writeln!(output, "keyboard.brightness={}", state.brightness.get()).unwrap();
    render_keyboard_lighting(output, state);
    writeln!(
        output,
        "keyboard.backlight_timeout={}",
        state.backlight_timeout
    )
    .unwrap();
    writeln!(output, "keyboard.sticky_keys={}", state.sticky_keys).unwrap();
    writeln!(output, "keyboard.win_menu_lock={}", state.win_menu_lock).unwrap();
}

fn render_keyboard_lighting(output: &mut String, state: &KeyboardState) {
    match state.lighting.mode {
        LightingMode::Static => {
            writeln!(output, "keyboard.lighting.mode=static").unwrap();
            render_zones(
                output,
                "keyboard.lighting.settings.static.zones",
                &state.lighting.static_zones,
            );
        }
        LightingMode::Dynamic => {
            let effect = dynamic_effect_name(state.lighting.dynamic.effect);
            writeln!(output, "keyboard.lighting.mode={effect}").unwrap();
            render_dynamic_lighting(
                output,
                &format!("keyboard.lighting.settings.{effect}"),
                state.lighting.dynamic,
            );
        }
    }
}

fn render_zones(output: &mut String, prefix: &str, zones: &[crate::device::Zone; 4]) {
    for (index, zone) in zones.iter().enumerate() {
        writeln!(output, "{prefix}[{}].enabled={}", index + 1, zone.enabled).unwrap();
        writeln!(output, "{prefix}[{}].color={}", index + 1, zone.color).unwrap();
    }
}

fn render_dynamic_lighting(output: &mut String, prefix: &str, lighting: DynamicLighting) {
    match lighting.effect {
        DynamicEffect::Breathing { color } => {
            writeln!(output, "{prefix}.color={color}").unwrap();
            writeln!(output, "{prefix}.speed={}", lighting.speed.get()).unwrap();
        }
        DynamicEffect::Neon => {
            writeln!(output, "{prefix}.speed={}", lighting.speed.get()).unwrap();
        }
        DynamicEffect::Shifting { color, direction } => {
            writeln!(output, "{prefix}.color={color}").unwrap();
            writeln!(output, "{prefix}.speed={}", lighting.speed.get()).unwrap();
            writeln!(output, "{prefix}.direction={}", direction_name(direction)).unwrap();
        }
        DynamicEffect::Wave { direction } => {
            writeln!(output, "{prefix}.speed={}", lighting.speed.get()).unwrap();
            writeln!(output, "{prefix}.direction={}", direction_name(direction)).unwrap();
        }
        DynamicEffect::Zoom { color } => {
            writeln!(output, "{prefix}.color={color}").unwrap();
            writeln!(output, "{prefix}.speed={}", lighting.speed.get()).unwrap();
        }
    }
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
        settings: (state.mode == FanMode::Custom).then(|| FanSettingsJson {
            custom: FanCustomJson {
                cpu: fan_custom_control_json(state.custom.cpu),
                gpu: fan_custom_control_json(state.custom.gpu),
            },
        }),
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
        FanCustomControl::Auto { .. } => FanCustomControlJson {
            mode: "auto",
            percent: None,
        },
        FanCustomControl::Manual { percent } => FanCustomControlJson {
            mode: "manual",
            percent: Some(percent.get()),
        },
    }
}

fn keyboard_json(state: &KeyboardState) -> KeyboardJson {
    KeyboardJson {
        brightness: state.brightness.get(),
        lighting: LightingJson {
            mode: active_lighting_mode_name(state),
            settings: lighting_settings_json(state),
        },
        backlight_timeout: state.backlight_timeout,
        sticky_keys: state.sticky_keys,
        win_menu_lock: state.win_menu_lock,
    }
}

fn lighting_settings_json(state: &KeyboardState) -> LightingSettingsJson {
    match state.lighting.mode {
        LightingMode::Static => LightingSettingsJson::Static {
            zones: state.lighting.static_zones.map(|zone| ZoneJson {
                enabled: zone.enabled,
                color: zone.color,
            }),
        },
        LightingMode::Dynamic => match state.lighting.dynamic.effect {
            DynamicEffect::Breathing { color } => LightingSettingsJson::Breathing {
                color,
                speed: state.lighting.dynamic.speed.get(),
            },
            DynamicEffect::Neon => LightingSettingsJson::Neon {
                speed: state.lighting.dynamic.speed.get(),
            },
            DynamicEffect::Shifting { color, direction } => LightingSettingsJson::Shifting {
                color,
                speed: state.lighting.dynamic.speed.get(),
                direction,
            },
            DynamicEffect::Wave { direction } => LightingSettingsJson::Wave {
                speed: state.lighting.dynamic.speed.get(),
                direction,
            },
            DynamicEffect::Zoom { color } => LightingSettingsJson::Zoom {
                color,
                speed: state.lighting.dynamic.speed.get(),
            },
        },
    }
}

fn active_lighting_mode_name(state: &KeyboardState) -> &'static str {
    match state.lighting.mode {
        LightingMode::Static => "static",
        LightingMode::Dynamic => dynamic_effect_name(state.lighting.dynamic.effect),
    }
}

fn dynamic_effect_name(effect: DynamicEffect) -> &'static str {
    match effect {
        DynamicEffect::Breathing { .. } => "breathing",
        DynamicEffect::Neon => "neon",
        DynamicEffect::Shifting { .. } => "shifting",
        DynamicEffect::Wave { .. } => "wave",
        DynamicEffect::Zoom { .. } => "zoom",
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
        Brightness, DynamicSpeed, FanChange, FanCustomControl, FanCustomState, FanTelemetry,
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
    fn fan_text_renders_only_active_custom_settings() {
        let output = render_text(StatusValue::Fan(sample_state().fan));
        assert_eq!(
            output,
            "fan.mode=custom\n\
fan.cpu.temperature_c=60\n\
fan.cpu.rpm=2400\n\
fan.gpu.temperature_c=45\n\
fan.gpu.rpm=2300\n\
fan.settings.custom.cpu.mode=manual\n\
fan.settings.custom.cpu.percent=70\n\
fan.settings.custom.gpu.mode=auto\n"
        );
    }

    #[test]
    fn fan_mutations_render_only_changed_fields() {
        assert_eq!(render_fan_mode_text(FanMode::Max), "fan.mode=max\n");

        let request = FanCustomRequest::new(
            Some(FanChange::Manual(Percent::new(70).unwrap())),
            Some(FanChange::Auto),
        )
        .unwrap();
        assert_eq!(
            render_fan_custom_text(&sample_state().fan, request),
            "fan.mode=custom\n\
fan.settings.custom.cpu.mode=manual\n\
fan.settings.custom.cpu.percent=70\n\
fan.settings.custom.gpu.mode=auto\n"
        );
        let output = render_fan_custom_text(&sample_state().fan, request);
        assert!(!output.contains("temperature_c"));
        assert!(!output.contains(".rpm="));
    }

    #[test]
    fn keyboard_text_renders_only_active_static_settings() {
        let output = render_text(StatusValue::Keyboard(sample_state().keyboard));
        assert!(output.contains("keyboard.lighting.mode=static\n"));
        assert!(output.contains("keyboard.lighting.settings.static.zones[1].enabled=true\n"));
        assert!(output.contains("keyboard.lighting.settings.static.zones[4].color=#FFFFFF\n"));
        assert!(!output.contains("keyboard.lighting.mode=dynamic\n"));
        assert!(!output.contains("keyboard.lighting.settings.wave"));
        assert!(!output.contains("keyboard.lighting.static"));
        assert!(!output.contains("keyboard.lighting.dynamic"));
    }

    #[test]
    fn keyboard_text_renders_active_wave_settings_without_color() {
        let mut state = sample_state().keyboard;
        state.lighting.mode = LightingMode::Dynamic;
        let output = render_text(StatusValue::Keyboard(state));
        assert!(output.contains("keyboard.lighting.mode=wave\n"));
        assert!(output.contains("keyboard.lighting.settings.wave.speed=3\n"));
        assert!(output.contains("keyboard.lighting.settings.wave.direction=fromleft\n"));
        assert!(!output.contains("keyboard.lighting.settings.wave.color"));
        assert!(!output.contains("keyboard.lighting.settings.static"));
    }

    #[test]
    fn keyboard_mutations_render_only_changed_fields() {
        let state = sample_state().keyboard;
        assert_eq!(
            render_keyboard_brightness_text(&state),
            "keyboard.brightness=5\n"
        );
        assert_eq!(
            render_keyboard_timeout_text(&state),
            "keyboard.backlight_timeout=true\n"
        );
        assert_eq!(
            render_keyboard_sticky_keys_text(&state),
            "keyboard.sticky_keys=false\n"
        );
        assert_eq!(
            render_keyboard_win_menu_lock_text(&state),
            "keyboard.win_menu_lock=true\n"
        );

        let static_output = render_keyboard_lighting_text(&state);
        assert!(static_output.starts_with("keyboard.lighting.mode=static\n"));
        assert!(
            static_output.contains("keyboard.lighting.settings.static.zones[4].color=#FFFFFF\n")
        );
        assert!(!static_output.contains("keyboard.brightness"));
        assert!(!static_output.contains("keyboard.backlight_timeout"));

        let mut dynamic_state = state;
        dynamic_state.lighting.mode = LightingMode::Dynamic;
        let dynamic_output = render_keyboard_lighting_text(&dynamic_state);
        assert_eq!(
            dynamic_output,
            "keyboard.lighting.mode=wave\n\
keyboard.lighting.settings.wave.speed=3\n\
keyboard.lighting.settings.wave.direction=fromleft\n"
        );
        assert!(!dynamic_output.contains("keyboard.lighting.settings.static"));
        assert!(!dynamic_output.contains("keyboard.brightness"));
    }

    fn dynamic_text(effect: DynamicEffect) -> String {
        let mut state = sample_state().keyboard;
        state.lighting.mode = LightingMode::Dynamic;
        state.lighting.dynamic.effect = effect;
        render_text(StatusValue::Keyboard(state))
    }

    #[test]
    fn dynamic_text_uses_only_effect_supported_fields() {
        let breathing = dynamic_text(DynamicEffect::Breathing {
            color: Rgb::parse("FF0000").unwrap(),
        });
        assert!(breathing.contains("keyboard.lighting.mode=breathing\n"));
        assert!(breathing.contains("keyboard.lighting.settings.breathing.color=#FF0000\n"));
        assert!(breathing.contains("keyboard.lighting.settings.breathing.speed=3\n"));
        assert!(!breathing.contains("direction"));

        let neon = dynamic_text(DynamicEffect::Neon);
        assert!(neon.contains("keyboard.lighting.mode=neon\n"));
        assert!(neon.contains("keyboard.lighting.settings.neon.speed=3\n"));
        assert!(!neon.contains(".color="));
        assert!(!neon.contains(".direction="));

        let shifting = dynamic_text(DynamicEffect::Shifting {
            color: Rgb::parse("00FF00").unwrap(),
            direction: Direction::FromRight,
        });
        assert!(shifting.contains("keyboard.lighting.mode=shifting\n"));
        assert!(shifting.contains("keyboard.lighting.settings.shifting.color=#00FF00\n"));
        assert!(shifting.contains("keyboard.lighting.settings.shifting.direction=fromright\n"));

        let zoom = dynamic_text(DynamicEffect::Zoom {
            color: Rgb::parse("0000FF").unwrap(),
        });
        assert!(zoom.contains("keyboard.lighting.mode=zoom\n"));
        assert!(zoom.contains("keyboard.lighting.settings.zoom.color=#0000FF\n"));
        assert!(!zoom.contains("direction"));
    }

    #[test]
    fn full_json_contains_only_active_mode_scoped_paths() {
        let value = serde_json::from_str::<serde_json::Value>(
            &all_json(StatusValue::All(sample_state()), false).unwrap(),
        )
        .unwrap();
        assert_eq!(value["fan"]["settings"]["custom"]["cpu"]["mode"], "manual");
        assert_eq!(value["fan"]["settings"]["custom"]["cpu"]["percent"], 70);
        assert_eq!(value["fan"]["settings"]["custom"]["gpu"]["mode"], "auto");
        assert!(
            value["fan"]["settings"]["custom"]["gpu"]
                .get("percent")
                .is_none()
        );
        assert!(value["fan"].get("custom").is_none());
        assert!(value["fan"]["cpu"].get("control").is_none());
        assert!(value["fan"]["cpu"].get("effective_control").is_none());
        assert_eq!(
            value["keyboard"]["lighting"]["settings"]["static"]["zones"]
                .as_array()
                .unwrap()
                .len(),
            4
        );
        assert_eq!(value["keyboard"]["lighting"]["mode"], "static");
        assert!(
            value["keyboard"]["lighting"]["settings"]
                .get("wave")
                .is_none()
        );
        assert!(value["keyboard"]["lighting"].get("static").is_none());
        assert!(value["keyboard"]["lighting"].get("dynamic").is_none());
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
    fn inactive_fan_settings_are_not_rendered() {
        let mut state = sample_state();
        state.fan.mode = FanMode::Auto;

        let value = serde_json::from_str::<serde_json::Value>(
            &all_json(StatusValue::All(state), false).unwrap(),
        )
        .unwrap();
        assert_eq!(value["fan"]["mode"], "auto");
        assert!(value["fan"].get("settings").is_none());
        assert!(value["fan"].get("custom").is_none());
    }

    #[test]
    fn dynamic_json_uses_effect_specific_settings() {
        let mut state = sample_state();
        state.keyboard.lighting.mode = LightingMode::Dynamic;
        let value = serde_json::from_str::<serde_json::Value>(
            &target_json(StatusValue::Keyboard(state.keyboard), false).unwrap(),
        )
        .unwrap();
        assert_eq!(value["lighting"]["mode"], "wave");
        assert_eq!(value["lighting"]["settings"]["wave"]["speed"], 3);
        assert_eq!(
            value["lighting"]["settings"]["wave"]["direction"],
            "from_left"
        );
        assert!(value["lighting"]["settings"]["wave"].get("color").is_none());
        assert!(value["lighting"]["settings"].get("static").is_none());
    }
}
