use crate::error::Result;
use crate::platform::{pipe, registry};
use anyhow::bail;
use serde::Serialize;

const CMD_SET_GAMING_PROFILE: u16 = 9;
const CMD_GET_GAMING_PROFILE: u16 = 10;
const CMD_WMI_SET_FUNCTION: u16 = 17;
const CMD_WMI_GET_FUNCTION: u16 = 20;
const QUERY_GAMING_PROFILE: u32 = 0;
const LCD_OVERDRIVE_SELECTOR: u64 = 0x10;
const LCD_OVERDRIVE_ENABLED_BIT: u64 = 1 << 48;
const DEFAULT_BK_HOTKEY_NUMBER: u32 = 132;
const TIMEOUT_SECONDS: u8 = 30;

#[derive(Debug, Clone, Serialize)]
pub struct DisplayState {
    pub overdrive_supported: Option<bool>,
    pub overdrive_live: Option<bool>,
}

pub fn set_overdrive(enabled: bool) -> Result<()> {
    if get_lcd_overdrive_supported() == Some(false) {
        bail!("LCD Overdrive is disabled by NitroSense support registry");
    }
    let mut payload = LCD_OVERDRIVE_SELECTOR;
    if enabled {
        payload |= LCD_OVERDRIVE_ENABLED_BIT;
    }
    let (raw, return_code) = pipe::service_set(CMD_SET_GAMING_PROFILE, &[pipe::u64_arg(payload)])?;
    ensure_success(CMD_SET_GAMING_PROFILE, &raw, return_code)
}

pub fn set_backlight_timeout(enabled: bool) -> Result<()> {
    let bk_hotkey_number = registry::read_hklm_dword_default(
        registry::NITROSENSE,
        "BK_Hotkey_Number",
        DEFAULT_BK_HOTKEY_NUMBER,
    );
    let live = read_backlight_raw(bk_hotkey_number)?;
    let timeout_seconds = if enabled { TIMEOUT_SECONDS } else { 0 };
    let payload =
        build_backlight_set_payload(bk_hotkey_number, live.brightness_percent, timeout_seconds);
    let (raw, return_code) = pipe::service_set(CMD_WMI_SET_FUNCTION, &[pipe::u64_arg(payload)])?;
    ensure_success(CMD_WMI_SET_FUNCTION, &raw, return_code)
}

pub fn read_state() -> DisplayState {
    DisplayState {
        overdrive_supported: get_lcd_overdrive_supported(),
        overdrive_live: read_overdrive().ok(),
    }
}

pub fn read_overdrive() -> Result<bool> {
    let (_, value) = pipe::service_get_u64(
        CMD_GET_GAMING_PROFILE,
        &[pipe::u32_arg(QUERY_GAMING_PROFILE)],
    )?;
    Ok(((value >> 48) & 0xFF) == 1)
}

pub fn read_backlight_timeout() -> Result<bool> {
    let bk_hotkey_number = registry::read_hklm_dword_default(
        registry::NITROSENSE,
        "BK_Hotkey_Number",
        DEFAULT_BK_HOTKEY_NUMBER,
    );
    Ok(read_backlight_raw(bk_hotkey_number)?.timeout_seconds == TIMEOUT_SECONDS)
}

fn get_lcd_overdrive_supported() -> Option<bool> {
    registry::read_hklm_dword(registry::ADVANCED_SETTINGS, "LCD_Overdrive_support")
        .ok()
        .map(|value| value != 0)
}

#[derive(Debug)]
struct BacklightRaw {
    brightness_percent: u8,
    timeout_seconds: u8,
}

fn read_backlight_raw(bk_hotkey_number: u32) -> Result<BacklightRaw> {
    let (_, value) = pipe::service_get_u64(
        CMD_WMI_GET_FUNCTION,
        &[pipe::u32_arg(build_backlight_get_payload(bk_hotkey_number))],
    )?;
    Ok(BacklightRaw {
        brightness_percent: ((value >> 32) & 0xFF) as u8,
        timeout_seconds: ((value >> 40) & 0xFF) as u8,
    })
}

fn build_backlight_get_payload(bk_hotkey_number: u32) -> u32 {
    1 | (bk_hotkey_number << 8) | 0x80000
}

fn build_backlight_set_payload(
    bk_hotkey_number: u32,
    brightness_percent: u8,
    timeout_seconds: u8,
) -> u64 {
    let mut payload = (2 | (bk_hotkey_number << 8) | 0x80000) as u64;
    payload |= (brightness_percent as u64) << 32;
    payload |= (timeout_seconds as u64) << 40;
    payload
}

fn ensure_success(cmd: u16, raw: &[u8], return_code: u32) -> Result<()> {
    if return_code != 0 {
        bail!(
            "PredatorSense command {cmd} failed with return_code={return_code} reply={}",
            hex(raw)
        );
    }
    Ok(())
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_backlight_timeout_payload() {
        let payload = build_backlight_set_payload(132, 100, 30);
        assert_eq!(payload & 0xFF, 2);
        assert_eq!((payload >> 32) & 0xFF, 100);
        assert_eq!((payload >> 40) & 0xFF, 30);
    }
}
