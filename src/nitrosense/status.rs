use crate::cli::StatusTarget;
use crate::nitrosense::{display, fan, keyboard, mode, sound};
use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Clone, Serialize)]
pub struct AppStatus {
    pub fan: Option<fan::FanState>,
    pub keyboard: KeyboardStatus,
    pub mode: Option<String>,
    pub display: DisplayStatus,
    pub sound: Option<sound::SoundState>,
}

#[derive(Debug, Clone, Serialize)]
pub struct KeyboardStatus {
    pub lighting: Option<keyboard::KeyboardState>,
    pub backlight_timeout: Option<bool>,
    pub sticky: Option<bool>,
    pub win_menu: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DisplayStatus {
    pub overdrive: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum StatusOutput {
    All(AppStatus),
    Fan(Option<fan::FanState>),
    Keyboard(KeyboardStatus),
    Mode(Option<String>),
    Display(DisplayStatus),
    Sound(Option<sound::SoundState>),
}

pub fn read_status(target: Option<StatusTarget>) -> StatusOutput {
    match target {
        None => StatusOutput::All(AppStatus {
            fan: read_fan(),
            keyboard: read_keyboard(),
            mode: read_mode(),
            display: read_display(),
            sound: read_sound(),
        }),
        Some(StatusTarget::Fan) => StatusOutput::Fan(read_fan()),
        Some(StatusTarget::Keyboard) => StatusOutput::Keyboard(read_keyboard()),
        Some(StatusTarget::Mode) => StatusOutput::Mode(read_mode()),
        Some(StatusTarget::Display) => StatusOutput::Display(read_display()),
        Some(StatusTarget::Sound) => StatusOutput::Sound(read_sound()),
    }
}

pub fn print_text(status: &StatusOutput, target: Option<StatusTarget>) {
    let value = serde_json::to_value(status).unwrap_or(Value::Null);
    if let Some(target) = target {
        print_value_text(&target.to_string(), &value);
        return;
    }

    if let Value::Object(values) = value {
        for (name, value) in values {
            print_value_text(&name, &value);
        }
    } else {
        print_value_text("status", &value);
    }
}

pub fn print_state_text(prefix: &str, state: &impl Serialize) -> serde_json::Result<()> {
    let value = serde_json::to_value(state)?;
    print_value_text(prefix, &value);
    Ok(())
}

fn read_fan() -> Option<fan::FanState> {
    fan::read_state().ok()
}

fn read_keyboard() -> KeyboardStatus {
    KeyboardStatus {
        lighting: keyboard::read_state().ok(),
        backlight_timeout: display::read_backlight_timeout().ok(),
        sticky: keyboard::read_sticky_keys()
            .ok()
            .map(|state| state.nitrosense_enabled),
        win_menu: keyboard::read_win_menu_lock().ok(),
    }
}

fn read_mode() -> Option<String> {
    mode::read_state().ok().map(|state| state.mode)
}

fn read_display() -> DisplayStatus {
    DisplayStatus {
        overdrive: display::read_state().overdrive_live,
    }
}

fn read_sound() -> Option<sound::SoundState> {
    sound::read_state().ok()
}

fn print_value_text(prefix: &str, value: &Value) {
    match value {
        Value::Object(values) => {
            for (name, value) in values {
                let child_prefix = format!("{prefix}.{name}");
                print_value_text(&child_prefix, value);
            }
        }
        Value::Array(_) => println!(
            "{prefix}={}",
            serde_json::to_string(value).unwrap_or_else(|_| "null".to_string())
        ),
        Value::String(value) => println!("{prefix}={value}"),
        Value::Number(value) => println!("{prefix}={value}"),
        Value::Bool(value) => println!("{prefix}={value}"),
        Value::Null => println!("{prefix}=null"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_serialization_contains_state_only() {
        let sound = sound::SoundState {
            preset: "music".to_string(),
            mode_code: 0,
        };
        let fan = fan::FanState {
            cpu_temperature_c: 60,
            gpu_temperature_c: 45,
            cpu_fan_rpm: 2400,
            gpu_fan_rpm: 2300,
            mode: "auto".to_string(),
            custom: None,
        };
        let json = serde_json::to_string(&StatusOutput::All(AppStatus {
            fan: Some(fan),
            keyboard: KeyboardStatus {
                lighting: None,
                backlight_timeout: Some(true),
                sticky: Some(false),
                win_menu: Some(true),
            },
            mode: Some("default".to_string()),
            display: DisplayStatus {
                overdrive: Some(false),
            },
            sound: Some(sound),
        }))
        .unwrap();

        assert!(json.contains("\"mode\":\"auto\""));
        assert!(json.contains("\"preset\":\"music\""));
        assert!(!json.contains("source"));
        assert!(!json.contains("reliability"));
        assert!(!json.contains("mode_code"));
        assert!(!json.contains("query"));
    }
}
