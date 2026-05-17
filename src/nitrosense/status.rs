use crate::nitrosense::{display, fan, keyboard, mode, sound};
use crate::platform::registry;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Status<T: Serialize> {
    pub value: T,
    pub source: String,
    pub reliability: Reliability,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Reliability {
    Live,
    Validated,
    Partial,
    Persisted,
    Unavailable,
}

#[derive(Debug, Clone, Serialize)]
pub struct AppStatus {
    pub status_model: String,
    pub fan: FanStatus,
    pub keyboard: KeyboardStatus,
    pub mode: ModeStatus,
    pub display: DisplayStatus,
    pub sound: SoundStatus,
}

#[derive(Debug, Clone, Serialize)]
pub struct FanStatus {
    pub state: Status<Option<fan::FanState>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct KeyboardStatus {
    pub persisted_state: Status<Option<keyboard::KeyboardState>>,
    pub live_zone_status: Status<Option<Vec<keyboard::ZoneState>>>,
    pub sticky_keys_live: Status<Option<bool>>,
    pub win_menu_key_lock_live: Status<Option<bool>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModeStatus {
    pub live_mode: Status<Option<mode::OperationModeState>>,
    pub persisted_mode_code: Status<Option<u32>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DisplayStatus {
    pub state: Status<display::DisplayState>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SoundStatus {
    pub live_preset: Status<Option<sound::SoundState>>,
}

pub fn read_status() -> AppStatus {
    let fan_state = fan::read_state();
    let keyboard_state = keyboard::read_state();
    let live_zones = keyboard::read_zone_statuses();
    let sticky = keyboard::read_sticky_keys();
    let win_menu = keyboard::read_win_menu_lock();
    let mode_state = mode::read_state();
    let persisted_mode =
        registry::read_hklm_dword(registry::OVERCLOCK, "CurrentOperationMode").ok();
    let sound_state = sound::read_state();

    AppStatus {
        status_model: "live_plus_nitrosense_persisted_state".to_string(),
        fan: FanStatus {
            state: status_from_result(
                fan_state,
                "service cmd 13 + HKLM NitroSense FanControl, validated against NitroSense fan UI and service writes",
                Reliability::Validated,
                Some("RPM and temperatures are read live from the PredatorSense service. Exact active fan mode is resolved from CurrentFanMode plus per-fan custom registry fields when CurrentFanMode=custom.".to_string()),
            ),
        },
        keyboard: KeyboardStatus {
            persisted_state: status_from_result(
                keyboard_state,
                "NitroSense system XML",
                Reliability::Persisted,
                Some("Brightness, RGB colors, and static/dynamic mode use NitroSense ProgramData XML.".to_string()),
            ),
            live_zone_status: status_from_result(
                live_zones,
                "service cmd 12",
                Reliability::Live,
                Some("Service cmd 12 low byte maps 1=enabled and 0=disabled for per-zone live status.".to_string()),
            ),
            sticky_keys_live: match sticky {
                Ok(value) => Status {
                    value: Some(value.nitrosense_enabled),
                    source: "Windows SystemParametersInfo".to_string(),
                    reliability: Reliability::Live,
                    note: None,
                },
                Err(error) => unavailable("Windows SystemParametersInfo", error.to_string()),
            },
            win_menu_key_lock_live: status_from_result(
                win_menu,
                "service cmd 10/query 0",
                Reliability::Live,
                None,
            ),
        },
        mode: ModeStatus {
            live_mode: status_from_result(
                mode_state,
                "service cmd 34/query 11",
                Reliability::Live,
                None,
            ),
            persisted_mode_code: Status {
                value: persisted_mode,
                source: "HKLM NitroSense Overclock".to_string(),
                reliability: Reliability::Persisted,
                note: Some("NitroSense persisted UI state.".to_string()),
            },
        },
        display: DisplayStatus {
            state: Status {
                value: display::read_state(),
                source: "service getters + HKLM NitroSense AdvanceSettings".to_string(),
                reliability: Reliability::Partial,
                note: Some("Backlight timeout is validated as an enable/disable feature. LCD overdrive support and broader display behavior still need further validation.".to_string()),
            },
        },
        sound: SoundStatus {
            live_preset: match sound_state {
                Ok(state) => {
                    let reliability = if state.reliability == "unavailable" {
                        Reliability::Unavailable
                    } else {
                        Reliability::Live
                    };
                    Status {
                        value: Some(state),
                        source: "admin-agent sound getter".to_string(),
                        reliability,
                        note: Some("DTS code 9 means unavailable, not a visible NitroSense preset.".to_string()),
                    }
                }
                Err(error) => unavailable("admin-agent sound getter", error.to_string()),
            },
        },
    }
}

pub fn print_text(status: &AppStatus) {
    println!("status_model={}", status.status_model);
    println!();
    println!("[fan]");
    print_item("state", &status.fan.state);
    println!();
    println!("[keyboard]");
    print_item("persisted_state", &status.keyboard.persisted_state);
    print_item("live_zone_status", &status.keyboard.live_zone_status);
    print_item("sticky_keys_live", &status.keyboard.sticky_keys_live);
    print_item(
        "win_menu_key_lock_live",
        &status.keyboard.win_menu_key_lock_live,
    );
    println!();
    println!("[mode]");
    print_item("live_mode", &status.mode.live_mode);
    print_item("persisted_mode_code", &status.mode.persisted_mode_code);
    println!();
    println!("[display]");
    print_item("state", &status.display.state);
    println!();
    println!("[sound]");
    print_item("live_preset", &status.sound.live_preset);
}

fn status_from_result<T: Serialize>(
    result: anyhow::Result<T>,
    source: &str,
    reliability: Reliability,
    note: Option<String>,
) -> Status<Option<T>> {
    match result {
        Ok(value) => Status {
            value: Some(value),
            source: source.to_string(),
            reliability,
            note,
        },
        Err(error) => unavailable(source, error.to_string()),
    }
}

fn unavailable<T: Serialize>(source: &str, note: String) -> Status<Option<T>> {
    Status {
        value: None,
        source: source.to_string(),
        reliability: Reliability::Unavailable,
        note: Some(note),
    }
}

fn print_item<T: Serialize>(name: &str, item: &Status<T>) {
    let value = serde_json::to_string(&item.value).unwrap_or_else(|_| "null".to_string());
    println!(
        "{name}={value} source={} reliability={:?}",
        item.source, item.reliability
    );
    if let Some(note) = &item.note {
        println!("{name}_note={note}");
    }
}
