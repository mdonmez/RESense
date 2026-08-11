use crate::cli::StatusTarget;
use crate::device::{
    Direction, DynamicEffect, FanControl, FanMode, FanState, KeyboardState, LightingState,
    OperationMode, Rgb, SoundPreset, SystemState,
};
use crate::error::Result;
use serde::Serialize;

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
    cpu: FanChannelJson,
    gpu: FanChannelJson,
}

#[derive(Debug, Serialize)]
struct FanChannelJson {
    temperature_c: u16,
    rpm: u16,
    control: FanControlJson,
}

#[derive(Debug, Serialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
enum FanControlJson {
    Auto,
    Manual { percent: u8 },
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
    zones: [ZoneJson; 4],
    #[serde(skip_serializing_if = "Option::is_none")]
    effect: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    speed: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    color: Option<Rgb>,
    #[serde(skip_serializing_if = "Option::is_none")]
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
    match value {
        StatusValue::All(state) => {
            print_fan(&state.fan);
            print_keyboard(&state.keyboard);
            println!("mode={}", mode_name(state.mode));
            println!(
                "display.overdrive={}",
                optional_bool(state.display_overdrive)
            );
            println!("sound={}", optional_sound(state.sound));
        }
        StatusValue::Fan(state) => print_fan(&state),
        StatusValue::Keyboard(state) => print_keyboard(&state),
        StatusValue::Mode(mode) => println!("mode={}", mode_name(mode)),
        StatusValue::Display(value) => println!("display.overdrive={}", optional_bool(value)),
        StatusValue::Sound(value) => println!("sound={}", optional_sound(value)),
    }
}

pub fn print_fan_state(state: FanState) {
    print_fan(&state)
}

pub fn print_keyboard_state(state: KeyboardState) {
    print_keyboard(&state)
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

fn print_fan(state: &FanState) {
    println!("fan.mode={}", fan_mode_name(state.mode));
    print_fan_channel("fan.cpu", &state.cpu);
    print_fan_channel("fan.gpu", &state.gpu);
}

fn print_fan_channel(prefix: &str, channel: &crate::device::FanReading) {
    println!("{prefix}.temperature_c={}", channel.temperature_c);
    println!("{prefix}.rpm={}", channel.rpm);
    println!("{prefix}.control.mode={}", channel.control.mode_name());
    if let Some(percent) = channel.control.percent() {
        println!("{prefix}.control.percent={}", percent.get());
    }
}

fn print_keyboard(state: &KeyboardState) {
    println!("keyboard.brightness={}", state.brightness.get());
    match state.lighting {
        LightingState::Static { zones } => {
            println!("keyboard.lighting.mode=static");
            print_zones(&zones);
        }
        LightingState::Dynamic { zones, effect } => {
            println!("keyboard.lighting.mode=dynamic");
            print_zones(&zones);
            let (name, color, direction) = dynamic_effect_text(effect.effect);
            println!("keyboard.lighting.effect={name}");
            println!("keyboard.lighting.speed={}", effect.speed.get());
            if let Some(color) = color {
                println!("keyboard.lighting.color={color}");
            }
            if let Some(direction) = direction {
                println!("keyboard.lighting.direction={direction}");
            }
        }
    }
    println!("keyboard.backlight_timeout={}", state.backlight_timeout);
    println!("keyboard.sticky_keys={}", state.sticky_keys);
    println!("keyboard.win_menu_lock={}", state.win_menu_lock);
}

fn print_zones(zones: &[crate::device::Zone; 4]) {
    for (index, zone) in zones.iter().enumerate() {
        println!(
            "keyboard.lighting.zone{}.enabled={}",
            index + 1,
            zone.enabled
        );
        println!("keyboard.lighting.zone{}.color={}", index + 1, zone.color);
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
        cpu: fan_channel_json(&state.cpu),
        gpu: fan_channel_json(&state.gpu),
    }
}

fn fan_channel_json(channel: &crate::device::FanReading) -> FanChannelJson {
    FanChannelJson {
        temperature_c: channel.temperature_c,
        rpm: channel.rpm,
        control: match channel.control {
            FanControl::Auto { .. } => FanControlJson::Auto,
            FanControl::Manual { percent } => FanControlJson::Manual {
                percent: percent.get(),
            },
        },
    }
}

fn keyboard_json(state: &KeyboardState) -> KeyboardJson {
    let (mode, zones, effect, speed, color, direction) = match state.lighting {
        LightingState::Static { zones } => ("static", zones, None, None, None, None),
        LightingState::Dynamic { zones, effect } => {
            let (effect_name, color, direction) = dynamic_effect_json(effect.effect);
            (
                "dynamic",
                zones,
                Some(effect_name),
                Some(effect.speed.get()),
                color,
                direction,
            )
        }
    };
    KeyboardJson {
        brightness: state.brightness.get(),
        lighting: LightingJson {
            mode,
            zones: zones.map(|zone| ZoneJson {
                enabled: zone.enabled,
                color: zone.color,
            }),
            effect,
            speed,
            color,
            direction,
        },
        backlight_timeout: state.backlight_timeout,
        sticky_keys: state.sticky_keys,
        win_menu_lock: state.win_menu_lock,
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
    use crate::device::{Brightness, FanReading, Percent, Zone};

    fn sample_state() -> SystemState {
        SystemState {
            fan: FanState {
                mode: FanMode::Custom,
                cpu: FanReading {
                    temperature_c: 60,
                    rpm: 2400,
                    control: FanControl::Manual {
                        percent: Percent::new(70).unwrap(),
                    },
                },
                gpu: FanReading {
                    temperature_c: 45,
                    rpm: 2300,
                    control: FanControl::Auto {
                        remembered_percent: Percent::new(50).unwrap(),
                    },
                },
            },
            keyboard: KeyboardState {
                brightness: Brightness::new(5).unwrap(),
                lighting: LightingState::Static {
                    zones: [
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
    fn full_json_is_the_small_state_contract() {
        let value = serde_json::from_str::<serde_json::Value>(
            &all_json(StatusValue::All(sample_state()), false).unwrap(),
        )
        .unwrap();
        assert_eq!(value["fan"]["cpu"]["control"]["mode"], "manual");
        assert_eq!(value["fan"]["cpu"]["control"]["percent"], 70);
        assert_eq!(value["fan"]["gpu"]["control"]["mode"], "auto");
        assert_eq!(
            value["keyboard"]["lighting"]["zones"]
                .as_array()
                .unwrap()
                .len(),
            4
        );
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
}
